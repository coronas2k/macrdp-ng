use std::time::{Duration, Instant};

use ironrdp_server::{KeyboardEvent, MouseEvent, RdpServerInputHandler};
use macrdp_input::{DEFAULT_WHEEL_DELTA, KeyboardInjector, MouseButton, MouseInjector};
use std::sync::{Arc, Mutex};

/// Pointer and scroll tuning (from server config).
#[derive(Debug, Clone)]
pub struct InputOptions {
    pub wheel_delta: i32,
    pub scroll_lines_per_notch: i32,
    pub pointer_invert_y: bool,
}

impl Default for InputOptions {
    fn default() -> Self {
        Self {
            wheel_delta: DEFAULT_WHEEL_DELTA,
            scroll_lines_per_notch: 1,
            pointer_invert_y: false,
        }
    }
}

/// Maps RDP desktop coordinates to macOS logical coordinates.
#[derive(Debug, Clone)]
pub struct MouseCoordMapper(Arc<Mutex<CoordState>>);

#[derive(Debug)]
struct CoordState {
    logical_w: f64,
    logical_h: f64,
    rdp_w: f64,
    rdp_h: f64,
    invert_y: bool,
}

impl MouseCoordMapper {
    pub fn new(logical_w: u16, logical_h: u16, rdp_w: u16, rdp_h: u16, invert_y: bool) -> Self {
        tracing::info!(
            logical_w, logical_h, rdp_w, rdp_h, invert_y,
            "MouseCoordMapper initialized"
        );
        Self(Arc::new(Mutex::new(CoordState {
            logical_w: logical_w as f64,
            logical_h: logical_h as f64,
            rdp_w: rdp_w.max(1) as f64,
            rdp_h: rdp_h.max(1) as f64,
            invert_y,
        })))
    }

    pub fn map(&self, rdp_x: u16, rdp_y: u16) -> (u16, u16) {
        let s = self.0.lock().unwrap();
        let mac_x = (rdp_x as f64 * s.logical_w / s.rdp_w).round() as u16;
        let mac_y = rdp_y as f64 * s.logical_h / s.rdp_h;
        let mac_y = if s.invert_y {
            (s.logical_h - mac_y).round()
        } else {
            mac_y.round()
        };
        (mac_x, mac_y.max(0.0) as u16)
    }

    pub fn update_rdp_size(&self, rdp_w: u16, rdp_h: u16) {
        let mut s = self.0.lock().unwrap();
        s.rdp_w = rdp_w.max(1) as f64;
        s.rdp_h = rdp_h.max(1) as f64;
        tracing::info!(
            new_rdp_w = rdp_w, new_rdp_h = rdp_h,
            logical_w = s.logical_w, logical_h = s.logical_h,
            "MouseCoordMapper: RDP desktop size updated"
        );
    }
}

/// Bridges RDP input events to macOS CGEvent injection
pub struct MacInputHandler {
    keyboard: Option<KeyboardInjector>,
    mouse: Option<MouseInjector>,
    last_mouse_x: u16,
    last_mouse_y: u16,
    coord_mapper: MouseCoordMapper,
    input_opts: InputOptions,
    left_down: bool,
    right_down: bool,
    last_left_press: Option<(Instant, u16, u16)>,
}

impl MacInputHandler {
    pub fn new(coord_mapper: MouseCoordMapper, input_opts: InputOptions) -> Self {
        let keyboard = KeyboardInjector::new()
            .map_err(|e| tracing::error!("Failed to create keyboard injector: {e}"))
            .ok();
        let mouse = MouseInjector::new()
            .map_err(|e| tracing::error!("Failed to create mouse injector: {e}"))
            .ok();

        if keyboard.is_none() || mouse.is_none() {
            tracing::warn!(
                "Input injection may fail — ensure Accessibility permission is granted"
            );
        }

        Self {
            keyboard,
            mouse,
            last_mouse_x: 0,
            last_mouse_y: 0,
            coord_mapper,
            input_opts,
            left_down: false,
            right_down: false,
            last_left_press: None,
        }
    }

    fn map_pointer(&mut self, x: u16, y: u16) -> (u16, u16) {
        let (mx, my) = self.coord_mapper.map(x, y);
        self.last_mouse_x = mx;
        self.last_mouse_y = my;
        (mx, my)
    }
}

impl RdpServerInputHandler for MacInputHandler {
    fn keyboard(&mut self, event: KeyboardEvent) {
        let Some(kb) = &self.keyboard else { return };

        let result = match event {
            KeyboardEvent::Pressed { code, extended } => kb.inject_key(code, extended, true),
            KeyboardEvent::Released { code, extended } => kb.inject_key(code, extended, false),
            KeyboardEvent::UnicodePressed(ch) => kb.inject_unicode(ch, true),
            KeyboardEvent::UnicodeReleased(ch) => kb.inject_unicode(ch, false),
            KeyboardEvent::Synchronize(_flags) => {
                tracing::debug!("Keyboard synchronize event (ignored)");
                Ok(())
            }
        };

        if let Err(e) = result {
            tracing::warn!("Keyboard injection failed: {e}");
        }
    }

    fn mouse(&mut self, event: MouseEvent) {
        let Some(m) = &self.mouse else { return };
        let wd = self.input_opts.wheel_delta;
        let lpn = self.input_opts.scroll_lines_per_notch;

        let result = match event {
            MouseEvent::Move { x, y } => {
                let (mx, my) = self.map_pointer(x, y);
                if self.left_down {
                    m.drag_to(MouseButton::Left, mx, my)
                } else if self.right_down {
                    m.drag_to(MouseButton::Right, mx, my)
                } else {
                    m.move_to(mx, my)
                }
            }
            MouseEvent::LeftPressed => {
                self.left_down = true;
                let (mx, my) = (self.last_mouse_x, self.last_mouse_y);
                let now = Instant::now();
                let click_count = if let Some((t, px, py)) = self.last_left_press {
                    if now.duration_since(t) < Duration::from_millis(400)
                        && (mx as i32 - px as i32).abs() <= 2
                        && (my as i32 - py as i32).abs() <= 2
                    {
                        2i64
                    } else {
                        1
                    }
                } else {
                    1
                };
                self.last_left_press = Some((now, mx, my));
                m.button_event_with_click_count(MouseButton::Left, true, mx, my, click_count)
            }
            MouseEvent::LeftReleased => {
                self.left_down = false;
                m.button_event(MouseButton::Left, false, self.last_mouse_x, self.last_mouse_y)
            }
            MouseEvent::RightPressed => {
                self.right_down = true;
                m.button_event(MouseButton::Right, true, self.last_mouse_x, self.last_mouse_y)
            }
            MouseEvent::RightReleased => {
                self.right_down = false;
                m.button_event(MouseButton::Right, false, self.last_mouse_x, self.last_mouse_y)
            }
            MouseEvent::MiddlePressed => {
                m.button_event(MouseButton::Middle, true, self.last_mouse_x, self.last_mouse_y)
            }
            MouseEvent::MiddleReleased => {
                m.button_event(MouseButton::Middle, false, self.last_mouse_x, self.last_mouse_y)
            }
            MouseEvent::VerticalScroll { value } => m.scroll_vertical(value, wd, lpn),
            MouseEvent::Scroll { x, y } => {
                let v = if y != 0 {
                    m.scroll_vertical(y as i16, wd, lpn)
                } else {
                    Ok(())
                };
                if v.is_err() {
                    v
                } else if x != 0 {
                    m.scroll_horizontal(x, wd, lpn)
                } else {
                    Ok(())
                }
            }
            MouseEvent::RelMove { x, y } => {
                let nx = (self.last_mouse_x as i32 + x).clamp(0, 65535) as u16;
                let ny = (self.last_mouse_y as i32 + y).clamp(0, 65535) as u16;
                self.last_mouse_x = nx;
                self.last_mouse_y = ny;
                if self.left_down {
                    m.drag_to(MouseButton::Left, nx, ny)
                } else {
                    m.move_to(nx, ny)
                }
            }
            other => {
                tracing::trace!(?other, "Unhandled mouse event");
                Ok(())
            }
        };

        if let Err(e) = result {
            tracing::warn!("Mouse injection failed: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coord_mapper_scales_proportionally() {
        let m = MouseCoordMapper::new(1920, 1080, 3840, 2160, false);
        let (x, y) = m.map(1920, 1080);
        assert_eq!(x, 960);
        assert_eq!(y, 540);
    }

    #[test]
    fn coord_mapper_invert_y() {
        let m = MouseCoordMapper::new(100, 100, 100, 100, true);
        let (_, y0) = m.map(50, 0);
        let (_, y1) = m.map(50, 100);
        assert!(y0 > y1);
    }
}
