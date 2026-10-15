# Improvement blueprint

Living backlog for MacRDP-Ng. Link fork issues to [x6nux/macrdp](https://github.com/x6nux/macrdp) upstream.

| Upstream | Title | Status (v0.2.0) |
|----------|-------|-----------------|
| [#1](https://github.com/x6nux/macrdp/issues/1) | Mouse scrolling too fast | Mitigated — wheel delta scaling |
| [#2](https://github.com/x6nux/macrdp/issues/2) | macOS 26 / GFX / build meta | Partial — build/tray/caps; frame glitch open |
| [#3](https://github.com/x6nux/macrdp/issues/3) | Compile on macOS Tahoe | Partial — toolchain docs + deployment target |
| [#4](https://github.com/x6nux/macrdp/issues/4) | Double-click not working | Mitigated — click-count synthesis |
| [#5](https://github.com/x6nux/macrdp/issues/5) | libswiftCore.dylib not found | Partial — CI binaries + no DYLD hack |
| [#6](https://github.com/x6nux/macrdp/issues/6) | Black screen MS RD macOS | Partial — `pointer_invert_y`; video TBD |
| [#7](https://github.com/x6nux/macrdp/issues/7) | Mouse drag and drop | Mitigated — drag events while held |

Fork GitHub Issues are disabled on `coronas2k/macrdp-ng`; track via upstream links and this blueprint.

## Shipped in v0.2.0 (`feat/improvements`)

- `docs/` engineering documentation
- PR CI (`.github/workflows/ci.yml`)
- macOS 14 deployment target; removed `DYLD_LIBRARY_PATH`; Tauri min OS; Regular activation policy
- Lenient GFX `CapabilitiesAdvertise` + V10_7 fallback
- Input: scroll scaling, `MouseEvent::Scroll`, drag, double-click, coord mapper tests, optional `pointer_invert_y`

## Remaining

- H.264/GFX corruption on some clients after sustained session (#2E)
- MS RD macOS black screen if not pointer-only (#6)
- Enable GitHub Issues on fork or use external tracker
