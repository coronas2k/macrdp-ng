# Features and functions

## Protocol and session

| Feature | Status | Notes |
|---------|--------|-------|
| Standard RDP (TCP) | Supported | Default port 3389 in code; examples use 13389 |
| TLS | Supported | PEM cert/key or auto-generated |
| NLA / CredSSP (Hybrid) | Supported | Windows mstsc credential prompt |
| RemoteFX / GFX | Supported | Via patched `ironrdp-server-gfx` |
| H.264 AVC420 | Supported | Default; broad client compatibility |
| H.264 AVC444 | Supported | `chroma_mode = "avc444"`; needs capable RDP 10+ client |
| GFX uncompressed dirty rects | Supported | Small region updates without re-encode |
| Idle disconnect | Supported | `idle_timeout_secs` (default 1800) |
| Dynamic resolution | Supported | Client resize updates mapper + GFX surface |
| Hot config (UI / core API) | Supported | Frame rate, bitrate, log level, cursor, resolution, encoder, chroma |

## Video capture and encoding

| Feature | Status | Notes |
|---------|--------|-------|
| ScreenCaptureKit capture | Supported | Primary path |
| Lock screen / CG fallback | Supported | When SCK cannot capture |
| HiDPI / Retina | Supported | `resolution`: `auto`, `WxH`, or legacy `2`/`3` scale |
| Hardware encode (VideoToolbox) | Supported | `encoder = "hardware"` |
| Software encode (OpenH264) | Supported | `encoder = "software"` |
| Quality presets | Supported | `low_latency`, `balanced`, `high_quality` |
| Adaptive bitrate | Supported | Optional `bitrate_mbps` ceiling |
| Show remote cursor | Supported | `show_cursor` (default true) |
| Performance stats CLI | Supported | `macrdp-server --perf` |

## Input

| Feature | Status | Notes |
|---------|--------|-------|
| Keyboard (104-key style) | Supported | Scancode mapping |
| Mouse move / buttons | Supported | CGEvent injection |
| Scroll wheel | Supported | See upstream issue #1 (speed) |
| Double-click | Reported broken | Upstream issue #4 |
| Drag and drop | Reported broken | Upstream issue #7 |

## Audio and clipboard

| Feature | Default | Config |
|---------|---------|--------|
| Audio forwarding | On | `[audio]` enabled, sample_rate, channels |
| Clipboard sync | On | `[clipboard]` enabled |
| Clipboard file transfer | On | `file_transfer`, `max_file_size_mb` |

## Desktop application (`macrdp-ui`)

| Area | Function |
|------|----------|
| Dashboard | Server status, FPS chart, metrics, active connections |
| Settings | Category navigation; config edit; restart when required |
| Permissions | Screen Recording + Accessibility guidance |
| Logs | Live log stream (Console-style) |
| Statistics | Time range, bar charts, summary cards (SQLite history) |
| Tray / popover | Menu bar icon; compact popover window |
| Autostart | Tauri autostart plugin |

## CLI-only extras

- `--perf` for shutdown performance summary
- Dual log sinks (stderr + `macrdp.log`) without UI bridge

## Client compatibility (observed / reported)

| Client | Reported experience |
|--------|---------------------|
| Windows mstsc | Often works; GFX glitches on macOS 26 per issue #2 |
| MS Remote Desktop (macOS) | Black screen + wrong mouse scale (issue #6) |
| FreeRDP (Linux) | Black screen reported (issue #6 comment) |
| Android MS Remote Desktop | Capability / protocol errors (issue #2) |
| Remmina (Linux) | Scroll too fast (issue #1) |

Treat compatibility as **client-specific** until issues are fixed; do not assume one client result generalizes.

## Non-goals (current code)

- macOS versions below 14
- Non-macOS server host
- Multi-tenant or directory authentication
- Built-in WAN relay or NAT traversal
