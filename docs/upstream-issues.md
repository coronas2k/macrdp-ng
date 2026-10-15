# Upstream issues (x6nux/macrdp)

All issues on https://github.com/x6nux/macrdp were **open** when this document was written. MacRDP-Ng should track these as the primary community backlog until fork-specific issues exist.

| # | Title | Theme |
|---|--------|--------|
| [1](https://github.com/x6nux/macrdp/issues/1) | Mouse Scrolling Error | Input — scroll wheel sensitivity |
| [2](https://github.com/x6nux/macrdp/issues/2) | macOS 26 compatibility / GFX frame misalignment | Build, tray, capabilities, video |
| [3](https://github.com/x6nux/macrdp/issues/3) | Can't Compile on macOS Tahoe 26.5.2 | Swift / screencapturekit build |
| [4](https://github.com/x6nux/macrdp/issues/4) | 鼠标双击打开文件夹不响应 | Input — double-click |
| [5](https://github.com/x6nux/macrdp/issues/5) | libswiftCore.dylib cannot be found | Intel Mac + Swift dyld paths |
| [6](https://github.com/x6nux/macrdp/issues/6) | Black screen with Microsoft Remote Desktop (macOS client) | Client-specific GFX + mouse mapping |
| [7](https://github.com/x6nux/macrdp/issues/7) | Mouse drag and drop not working | Input — drag / long press |

---

## Issue 1 — Mouse scrolling

**Symptom:** Scroll wheel jumps to top/bottom (Remmina on Linux → Mac).

**Likely area:** `macrdp-input/src/mouse.rs` — `CGEventCreateScrollWheelEvent2` with `SCROLL_UNIT_LINE`; delta scaling from RDP wheel events may be too large.

**Fork direction:** Normalize wheel delta; add client-specific scale factor or config knob.

---

## Issue 2 — macOS 26 / GFX (meta-issue)

Consolidates five problem classes:

### A. `DYLD_LIBRARY_PATH` in `.cargo/config.toml`

Forces Swift 5.5 `libswift_Concurrency.dylib`. On macOS 26: duplicate `SwiftNativeNSObject`, libdispatch assertion, **SIGTRAP in tray-icon**.

**Suggested fix:** Remove forced `DYLD_LIBRARY_PATH`; set `MACOSX_DEPLOYMENT_TARGET=14.0`.

**Repo state:** Still present in fork — high priority for MacRDP-Ng.

### B. Tauri deployment target

`tauri build` may emit binaries with min OS 11 and `@rpath/libswift_Concurrency.dylib` dependency.

**Suggested fix:** `bundle.macOS.minimumSystemVersion = "14.0"` in `tauri.conf.json`.

**Repo state:** Not set explicitly in current `tauri.conf.json`.

### C. `LSUIElement=true`

Agent-style app + NSStatusItem triggers same libdispatch path on macOS 26.

**Suggested fix:** Remove `LSUIElement` or upgrade tray stack; `set_activation_policy(Regular)` in setup.

**Repo state:** `macrdp-ui/src-tauri/Info.plist` still has `LSUIElement=true`.

### D. Unknown GFX CapabilitySet versions

`ironrdp-pdu` fails whole capabilities PDU when client sends V10_8+ sets → GFX never starts → timeout.

**Suggested fix:** Lenient decode in `crates/ironrdp-server-gfx/src/gfx.rs` — skip unknown caps, optional V10_7 fallback.

**Repo state:** Needs verification against current `gfx.rs`; issue author proposed patch approach.

### E. Frame rendering misalignment (unresolved upstream)

- mstsc: works briefly then corrupt frames
- Android: disconnect after keyframe + P-frame (`0x200d`)
- Encoder pads to 1280 width while surface 1272 — rectangles use 1272

**Likely area:** `macrdp-encode`, `ironrdp-server-gfx` GFX PDUs, `MacDisplay` keyframe sizing.

---

## Issue 3 — Compile on Tahoe 26.5.2

`screencapturekit` build script runs Swift; fails with linker / `PackageDescription` / Swift version mismatch.

Related to issues 2A and 5 — toolchain and `DYLD_*` environment.

---

## Issue 4 — Double-click

Finder/desktop double-click does not open folders; other input works.

**Likely area:** RDP click timing or missing double-click synthesis in `macrdp-input` / pointer handler.

---

## Issue 5 — libswiftCore.dylib (Intel + OCLP)

`rustc` aborts: `libswiftCore.dylib` not found under expected Swift 5.5 paths.

Workaround attempted: switch `DYLD_LIBRARY_PATH` → `DYLD_LIBRARY_FALLBACK_PATH`; then hits screencapturekit Swift `--scratch-path` error.

**Fork direction:** Document supported toolchains; fix capture crate build for modern Swift; provide prebuilt CI binaries (release workflow already uploads CLI).

---

## Issue 6 — Black screen (MS RD macOS client)

**Symptom:** Auth OK, video black; mouse moves in **top-left quarter** with **inverted Y**. Windows mstsc to **same server** works.

**Implication:** Client-specific capability/DPI negotiation or coordinate origin bug — not universal capture failure.

**Likely areas:**

- GFX/H.264 negotiation for MS RD macOS
- `MouseCoordMapper` vs client-reported desktop scale (issue mentions ~2× scale error)
- Y flip: top-left vs bottom-left origin in mapping

FreeRDP 3.31 on Linux also reported black screen (comment on #6).

---

## Issue 7 — Drag and drop

No drag; long press ineffective.

**Likely area:** Missing pointer down/move/up sequences or pressure events for macOS drag APIs.

---

## Suggested MacRDP-Ng priority order

1. **Build/runtime on macOS 15–26** — `.cargo/config.toml`, Tauri min OS, Swift capture build
2. **GFX capability forward compatibility** — issue #2D
3. **Client matrix** — MS RD macOS (#6), Android (#2E), mstsc corruption (#2E)
4. **Input polish** — scroll (#1), double-click (#4), drag (#7)
5. **CI** — PR checks + signed/notarized release policy

Link fork issues to upstream numbers when opening `coronas2k/macrdp-ng` tickets for traceability.
