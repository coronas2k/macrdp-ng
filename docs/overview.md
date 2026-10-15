# Overview

## What MacRDP-Ng is

**MacRDP-Ng** (`coronas2k/macrdp-ng`) is a community fork of **[x6nux/macrdp](https://github.com/x6nux/macrdp)** — a native **macOS RDP server** written in Rust. Remote clients (Windows mstsc, Microsoft Remote Desktop, FreeRDP, mobile clients) connect over standard RDP with GFX/H.264, NLA/CredSSP, and optional audio and clipboard channels.

The project ships two ways to run the server:

1. **CLI** — `macrdp-server` binary (`crates/macrdp-server`)
2. **Desktop app** — Tauri 2 + React UI (`macrdp-ui/`) that embeds the server via `macrdp-core`

## Lineage and maintenance

| Item | Detail |
|------|--------|
| Upstream | https://github.com/x6nux/macrdp |
| Fork | https://github.com/coronas2k/macrdp-ng |
| Upstream last commit (as of doc date) | `6be70bb` — 2026-05-18 — *resolution overhaul, GFX uncompressed path, hot config, log bridge* |
| Fork HEAD | Same commit as upstream at time of documentation (no divergent commits yet) |
| Upstream activity | No code commits for ~5 months; repo metadata updated more recently |
| Open upstream issues | 7 (all open) — see [upstream-issues.md](./upstream-issues.md) |

MacRDP-Ng exists to continue maintenance, fixes, and releases independently of upstream cadence.

## Platform and license

- **OS**: macOS 14+ (Sonoma); Apple Silicon primary; Intel builds in CI
- **Rust**: 1.75+ (workspace `rust-version`)
- **License**: GPLv3 ([LICENSE](../LICENSE))

## Repository layout

```
macrdp-ng/
├── crates/
│   ├── macrdp-server/          # CLI entry (main.rs)
│   ├── macrdp-core/            # Shared server library (UI + future embedders)
│   ├── macrdp-capture/         # ScreenCaptureKit + CoreGraphics fallback
│   ├── macrdp-input/           # Keyboard/mouse via CGEvent
│   ├── macrdp-encode/          # VideoToolbox + OpenH264
│   ├── macrdp-audio/           # System audio → RDP sound channel
│   ├── macrdp-clipboard/       # cliprdr + pasteboard/file transfer
│   ├── ironrdp-server-gfx/     # Patched ironrdp-server (GFX, AVC444, ZGFX)
│   └── ironrdp-acceptor-patched/  # Patched connection/CredSSP acceptor
├── macrdp-ui/                  # Tauri 2 + React/Vite console
├── .github/workflows/release.yml
├── Makefile                    # dev + release shortcuts
├── config.example.toml
└── docs/                       # This folder
```

## External dependencies (conceptual)

- **[IronRDP](https://github.com/Devolutions/IronRDP)** — RDP protocol (vendored patches in-tree)
- **ScreenCaptureKit** (`screencapturekit` crate) — primary capture; Swift build at compile time
- **Apple VideoToolbox** — hardware H.264
- **OpenH264** — software encoder path
- Design references: FreeRDP (AVC444), RustDesk (capture/input patterns)

## macOS permissions

| Permission | Purpose |
|------------|---------|
| Screen Recording | Capture display pixels |
| Accessibility | Inject keyboard and mouse |

Unsigned debug builds change binary identity; TCC may require re-authorization after each rebuild (documented in server startup checks).
