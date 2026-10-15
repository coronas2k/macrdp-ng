# MacRDP-Ng

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![macOS](https://img.shields.io/badge/macOS-14%2B-black.svg)](https://www.apple.com/macos/)

**Native RDP server for macOS.** Connect from Windows, Linux, iOS, or Android with any standard RDP client (mstsc, Microsoft Remote Desktop, FreeRDP, and others).

Community fork of [x6nux/macrdp](https://github.com/x6nux/macrdp), maintained at [coronas2k/macrdp-ng](https://github.com/coronas2k/macrdp-ng).

macOS has no built-in RDP host; VNC is often slower and softer. MacRDP-Ng targets a fast H.264/GFX remote desktop with NLA/TLS, optional audio and clipboard, and a menu-bar Tauri app or a CLI-only server.

---

## Requirements

| Requirement | Notes |
|-------------|--------|
| **macOS 14+** (Sonoma or later) | Deployment target for builds and bundles |
| **Rust stable ≥ 1.75** | Workspace `rust-version` |
| **Xcode or Command Line Tools** | Swift bridge for ScreenCaptureKit (`xcode-select --install`) |
| **Node.js 20** | Only for the desktop UI (`macrdp-ui`) |
| **Screen Recording** | System Settings → Privacy & Security |
| **Accessibility** | Required for keyboard and mouse injection |

Do **not** set `DYLD_LIBRARY_PATH` to Swift 5.5 back-deploy libraries; that breaks on newer macOS releases. See [docs/build-and-release.md](docs/build-and-release.md).

---

## What it does

### Protocol and session

- Standard RDP over TCP (default port **3389**; examples often use **13389**)
- TLS (auto-generated or custom PEM certificate and key)
- NLA / CredSSP (Hybrid) for Windows-style credential prompts
- RemoteFX / **GFX** with H.264 **AVC420** (default) or **AVC444** (RDP 10+ clients)
- GFX uncompressed dirty rectangles for small updates
- Dynamic resolution when the client resizes
- Idle disconnect (`idle_timeout_secs`, default 30 minutes)

### Video

- **ScreenCaptureKit** capture with **CoreGraphics** fallback on lock screen
- HiDPI: `resolution` = `auto`, `WxH` (e.g. `3840x2160`), or legacy numeric scale via `hidpi_scale` alias
- **VideoToolbox** (`hardware`) or **OpenH264** (`software` / `auto`)
- Quality presets: `low_latency`, `balanced`, `high_quality`
- Optional bitrate ceiling (`bitrate_mbps`), cursor in stream (`show_cursor`)
- Encoding optimizations: `skip_unchanged`, `idle_keyframe_sec`

### Input, audio, clipboard

- Keyboard (104-key style), mouse, scroll wheel (tunable scaling)
- Optional `pointer_invert_y` for clients with inverted Y
- Audio forwarding (`[audio]`) and clipboard sync / file transfer (`[clipboard]`)

### Desktop app (`macrdp-ui`)

Tauri 2 + React: dashboard, settings, permissions help, logs, statistics, menu-bar tray. The app embeds the same server logic as the CLI via `macrdp-core`. See [docs/desktop-ui.md](docs/desktop-ui.md).

Client compatibility varies by platform; see [docs/upstream-issues.md](docs/upstream-issues.md).

---

## Building

### CLI only

```bash
cargo build --release -p macrdp-server
./target/release/macrdp-server
```

### Desktop app (production)

```bash
cd macrdp-ui && npm ci && npm run tauri -- build
```

Artifacts:

- CLI: `target/release/macrdp-server`
- App: `macrdp-ui/src-tauri/target/release/bundle/macos/macrdp.app`
- DMG: `macrdp-ui/src-tauri/target/release/bundle/dmg/*.dmg`

### Makefile shortcuts

| Target | Purpose |
|--------|---------|
| `make install` | `npm install` in `macrdp-ui` |
| `make build-cli` | Build `macrdp-server` and symlink into Tauri tree |
| `make run-cli` | Run server (`cargo run -p macrdp-server`) |
| `make dev-ui` | CLI + `npm run tauri dev` |
| `make dev-front` | Vite dev server (port 1420) |
| `make check` | CLI + Tauri Rust + frontend production build |
| `make release` | Optimized Tauri build + DMG |
| `make clean` | `cargo clean` and UI build outputs |

Pre-built **draft** releases for tagged versions: [GitHub Releases](https://github.com/coronas2k/macrdp-ng/releases). Full CI and release pipeline: [docs/build-and-release.md](docs/build-and-release.md).

---

## Quick start

1. Copy [config.example.toml](config.example.toml) to `config.toml` (or use defaults).
2. Grant Screen Recording and Accessibility to the terminal or app binary.
3. Start the server:

```bash
cargo run --release -p macrdp-server
```

4. Connect from your RDP client to `your-mac-ip:<port>` (default **3389** if you did not change `port` in config; the example file uses **13389**).

If `username` / `password` are omitted, the server generates random credentials and prints them at startup.

---

## Configuration

Config file search order (first existing `config.toml` wins):

1. Current working directory (`./config.toml`)
2. `~/.config/macrdp/config.toml`
3. `~/Library/Application Support/macrdp/config.toml`

Override path with `--config /path/to/config.toml`.

### Server options (`config.toml`)

| Key | Default | Description |
|-----|---------|-------------|
| `port` | `3389` | Listen on `0.0.0.0:port` |
| `username` / `password` | none | NLA credentials; random if omitted |
| `width` / `height` | `0` | `0` = auto from display; non-zero forces desktop size |
| `frame_rate` | `60` | Target capture rate (`30`, `60`, `120`) |
| `resolution` | `auto` | `auto`, `WxH`, or legacy `hidpi_scale` (`2`–`4`) |
| `quality` | `high_quality` | `low_latency`, `balanced`, `high_quality` |
| `encoder` | `software` in schema; example uses `hardware` | `software`, `hardware`, `auto` |
| `chroma_mode` | `avc420` | `avc420` or `avc444` |
| `bitrate_mbps` | auto | Max target bitrate (Mbps) |
| `show_cursor` | `true` | Draw cursor in capture |
| `skip_unchanged` | `true` | Skip encode when frame unchanged |
| `idle_keyframe_sec` | `2` | IDR interval when screen is static |
| `idle_timeout_secs` | `1800` | Disconnect after no input (seconds) |
| `log_level` | `info` | `trace`, `debug`, `info`, `warn`, `error` |
| `cert_path` / `key_path` | auto-generated | TLS PEM files |
| `pointer_invert_y` | `false` | Flip RDP Y when mapping to CGEvent |
| `scroll_wheel_delta` | `120` | RDP wheel units per notch |
| `scroll_lines_per_notch` | `1` | macOS lines per notch after scaling |

**`[audio]`**

| Key | Default |
|-----|---------|
| `enabled` | `true` |
| `sample_rate` | `48000` |
| `channels` | `2` |

**`[clipboard]`**

| Key | Default |
|-----|---------|
| `enabled` | `true` |
| `file_transfer` | `true` |
| `max_file_size_mb` | `100` |

Commented reference: [config.example.toml](config.example.toml). Deeper detail: [docs/configuration.md](docs/configuration.md).

---

## CLI flags (`macrdp-server`)

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--port` | `-p` | `3389` | TCP listen port (overrides config when not default) |
| `--config` | `-c` | — | Path to `config.toml` |
| `--frame-rate` | — | — | Override `frame_rate` from config |
| `--log-level` | — | — | Override `log_level` (`trace` … `error`) |
| `--perf` | — | `false` | Collect performance stats; print summary on exit |

**Environment:** `RUST_LOG` (tracing filter) overrides `log_level` when set.

Example:

```bash
./target/release/macrdp-server -c ./config.toml --frame-rate 60 --perf
```

---

## Project layout

```
crates/
├── macrdp-server/          CLI binary
├── macrdp-core/              Shared server runtime (used by UI)
├── macrdp-capture/           ScreenCaptureKit + fallback
├── macrdp-input/             Keyboard and mouse injection
├── macrdp-encode/            H.264 encoding
├── macrdp-audio/             Audio channel
├── macrdp-clipboard/         Clipboard and file transfer
├── ironrdp-server-gfx/       Patched IronRDP GFX stack
└── ironrdp-acceptor-patched/ Connection acceptor
macrdp-ui/                    Tauri desktop application
```

Workspace patches `ironrdp-server` and `ironrdp-acceptor` to local crates. Architecture: [docs/architecture.md](docs/architecture.md).

---

## Documentation

| Doc | Contents |
|-----|----------|
| [docs/README.md](docs/README.md) | Index |
| [docs/features.md](docs/features.md) | Capabilities and limitations |
| [docs/build-and-release.md](docs/build-and-release.md) | Toolchains, Makefile, GitHub Actions |
| [docs/configuration.md](docs/configuration.md) | Hot config, UI mapping |
| [docs/upstream-issues.md](docs/upstream-issues.md) | Known client issues |
| [docs/improvement-blueprint.md](docs/improvement-blueprint.md) | Fork roadmap |

---

## Acknowledgments

- **[IronRDP](https://github.com/Devolutions/IronRDP)** — Rust RDP stack; GFX extensions in this fork
- **[FreeRDP](https://github.com/FreeRDP/FreeRDP)** — AVC444 reference behavior
- **[RustDesk](https://github.com/rustdesk/rustdesk)** — Inspiration for capture and input design

---

## License

**GNU General Public License v3.0** — see [LICENSE](LICENSE). Derivative works must remain under GPLv3.
