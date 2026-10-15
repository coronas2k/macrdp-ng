# Configuration

## Config file locations

Search order (first match wins):

1. `./config.toml` (current working directory)
2. `~/.config/macrdp/config.toml`
3. `~/Library/Application Support/macrdp/config.toml`

The CLI also accepts `--config /path/to/config.toml`.

The Tauri app maintains its own UI config (converted to `macrdp_core::ServerConfig` on start); it can align with the same fields.

## Server options (`ServerConfig`)

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `port` | u16 | 3389 | Listen address is `0.0.0.0:port` |
| `username` / `password` | string? | none | If omitted, random credentials printed at startup |
| `width` / `height` | u32 | 0 | 0 = auto from display; non-zero forces fixed desktop size |
| `frame_rate` | u32 | 60 | Target capture/encode rate |
| `resolution` | string? | `"auto"` | Also accepts legacy `hidpi_scale` alias: `auto`, `3840x2160`, or `2`–`4` scale |
| `quality` | string? | high_quality | `low_latency`, `balanced`, `high_quality` |
| `encoder` | string? | software in schema default; example uses hardware | `software`, `hardware`, `auto` |
| `chroma_mode` | string? | avc420 | `avc420` or `avc444` |
| `bitrate_mbps` | u32? | auto | Caps target bitrate |
| `show_cursor` | bool? | true | Draw cursor in capture |
| `idle_timeout_secs` | u64 | 1800 | Session idle disconnect |
| `log_level` | string? | info | trace … error |
| `cert_path` / `key_path` | path? | auto-generated | TLS PEM files |

### `[audio]`

| Key | Default |
|-----|---------|
| `enabled` | true |
| `sample_rate` | 48000 |
| `channels` | 2 |

### `[clipboard]`

| Key | Default |
|-----|---------|
| `enabled` | true |
| `file_transfer` | true |
| `max_file_size_mb` | 100 |

See [config.example.toml](../config.example.toml) and [config.zh.example.toml](../config.zh.example.toml) for commented templates.

## CLI (`macrdp-server`)

Defined in `crates/macrdp-server/src/config.rs` via `clap`:

- `--config` — optional config path
- `--ipc-socket` — Unix socket path for IPC control
- `--perf` — collect and print performance summary on shutdown

Environment:

- `RUST_LOG` / tracing filter overrides `log_level` when set

## Build environment (important)

`.cargo/config.toml` sets:

```toml
DYLD_LIBRARY_PATH = "... swift-5.5 ... libswift_Concurrency.dylib"
```

Required for some `screencapturekit` builds on older macOS; **problematic on macOS 26** — see [upstream-issues.md](./upstream-issues.md).

Local release signing (Makefile):

- `APPLE_SIGNING_IDENTITY` / `SIGN_IDENTITY := macrdp-dev`
- `MACRDP_CLI_PROFILE` passed to Tauri build for release vs debug CLI bundling

## Hot config (running server via `macrdp-core`)

Updates sent through `ServerHandle::update_config` / UI `set_config`:

| Update | Restart required? |
|--------|-------------------|
| Frame rate, bitrate, log level, show cursor, resolution, encoder, chroma | No (applied on channel) |
| Port, credentials, TLS paths | Yes (typical) |

UI `set_config` returns `restart_required` when a full restart is needed.
