use anyhow::Result;
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;
use foreign_types::ForeignType;

extern "C" {
    fn CGEventCreateScrollWheelEvent2(
        source: *mut core_graphics::sys::CGEventSource,
        units: u32,
        wheel_count: u32,
        wheel1: i32,
        wheel2: i32,
        wheel3: i32,
    ) -> *mut core_graphics::sys::CGEvent;
}

/// ScrollEventUnit::Line
const SCROLL_UNIT_LINE: u32 = 1;

/// Windows / RDP typical wheel notch delta.
pub const DEFAULT_WHEEL_DELTA: i32 = 120;

pub struct MouseInjector;

impl MouseInjector {
    pub fn new() -> Result<Self> {
        let _ = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| anyhow::anyhow!("Failed to create CGEventSource — check Accessibility permission"))?;
        Ok(Self)
    }

    fn source() -> Result<CGEventSource> {
        CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| anyhow::anyhow!("Failed to create CGEventSource"))
    }

    pub fn move_to(&self, x: u16, y: u16) -> Result<()> {
        self.pointer_event(CGEventType::MouseMoved, x, y, CGMouseButton::Left, 0)
    }

    pub fn drag_to(&self, button: MouseButton, x: u16, y: u16) -> Result<()> {
        let (event_type, cg_button) = match button {
            MouseButton::Left => (CGEventType::LeftMouseDragged, CGMouseButton::Left),
            MouseButton::Right => (CGEventType::RightMouseDragged, CGMouseButton::Right),
            MouseButton::Middle => (CGEventType::OtherMouseDragged, CGMouseButton::Center),
        };
        self.pointer_event(event_type, x, y, cg_button, 0)
    }

    pub fn button_event(&self, button: MouseButton, pressed: bool, x: u16, y: u16) -> Result<()> {
        self.button_event_with_click_count(button, pressed, x, y, 1)
    }

    pub fn button_event_with_click_count(
        &self,
        button: MouseButton,
        pressed: bool,
        x: u16,
        y: u16,
        click_count: i64,
    ) -> Result<()> {
        let (event_type, cg_button) = match (button, pressed) {
            (MouseButton::Left, true) => (CGEventType::LeftMouseDown, CGMouseButton::Left),
            (MouseButton::Left, false) => (CGEventType::LeftMouseUp, CGMouseButton::Left),
            (MouseButton::Right, true) => (CGEventType::RightMouseDown, CGMouseButton::Right),
            (MouseButton::Right, false) => (CGEventType::RightMouseUp, CGMouseButton::Right),
            (MouseButton::Middle, true) => (CGEventType::OtherMouseDown, CGMouseButton::Center),
            (MouseButton::Middle, false) => (CGEventType::OtherMouseUp, CGMouseButton::Center),
        };
        self.pointer_event(event_type, x, y, cg_button, click_count)
    }

    fn pointer_event(
        &self,
        event_type: CGEventType,
        x: u16,
        y: u16,
        cg_button: CGMouseButton,
        click_count: i64,
    ) -> Result<()> {
        let point = CGPoint::new(x as f64, y as f64);
        let source = Self::source()?;
        let event = CGEvent::new_mouse_event(source, event_type, point, cg_button)
            .map_err(|_| anyhow::anyhow!("Failed to create mouse event"))?;
        if click_count > 1 {
            // kCGMouseEventClickState
            event.set_integer_value_field(
                core_graphics::event::EventField::MOUSE_EVENT_CLICK_STATE,
                click_count,
            );
        }
        event.post(CGEventTapLocation::HID);
        tracing::trace!(?event_type, x, y, click_count, "Mouse pointer event");
        Ok(())
    }

    /// Scale RDP wheel units to macOS line scroll (negative = scroll down).
    pub fn scroll_vertical(&self, delta: i16, wheel_delta: i32, lines_per_notch: i32) -> Result<()> {
        let wheel_delta = wheel_delta.max(1);
        let lines_per_notch = lines_per_notch.max(1);
        let lines = ((delta as i32) / wheel_delta) * lines_per_notch;
        if lines == 0 {
            return Ok(());
        }
        self.post_scroll(lines, 0)
    }

    pub fn scroll_horizontal(&self, delta: i32, wheel_delta: i32, lines_per_notch: i32) -> Result<()> {
        let wheel_delta = wheel_delta.max(1);
        let lines_per_notch = lines_per_notch.max(1);
        let lines = (delta / wheel_delta) * lines_per_notch;
        if lines == 0 {
            return Ok(());
        }
        self.post_scroll(0, lines)
    }

    fn post_scroll(&self, vertical_lines: i32, horizontal_lines: i32) -> Result<()> {
        unsafe {
            let event_ref = CGEventCreateScrollWheelEvent2(
                std::ptr::null_mut(),
                SCROLL_UNIT_LINE,
                if horizontal_lines != 0 { 2 } else { 1 },
                vertical_lines,
                horizontal_lines,
                0,
            );
            if event_ref.is_null() {
                return Err(anyhow::anyhow!("Failed to create scroll event"));
            }
            let event = CGEvent::from_ptr(event_ref);
            event.post(CGEventTapLocation::HID);
        }
        tracing::trace!(vertical_lines, horizontal_lines, "Mouse scroll");
        Ok(())
    }

    /// Legacy: raw line count (avoid for RDP wheel deltas).
    pub fn scroll(&self, vertical: i16) -> Result<()> {
        self.scroll_vertical(vertical, 1, 1)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}
