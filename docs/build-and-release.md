# Build and release pipelines

## Prerequisites

| Tool | Version / notes |
|------|-----------------|
| macOS | 14+ for running; Xcode or CLT for Swift link (capture crate) |
| Rust | stable, ≥ 1.75 |
| Node.js | 20 (matches CI) |
| npm | `macrdp-ui/package-lock.json` |

Optional: codesign identity `macrdp-dev` (or change Makefile / `tauri.conf.json`).

## Makefile targets

| Target | Purpose |
|--------|---------|
| `make install` | `npm install` in `macrdp-ui` |
| `make build-cli` | `cargo build -p macrdp-server` + symlink binary into Tauri tree |
| `make run-cli` | Run server from workspace |
| `make run-cli-ipc` | Server with `--ipc-socket /tmp/macrdp.sock` |
| `make dev-ui` | Build CLI + `npm run tauri dev` |
| `make dev-front` | Vite only on port 1420 |
| `make check` | CLI + Tauri Rust + frontend production build |
| `make release` | Optimized Tauri build + DMG (`MACRDP_CLI_PROFILE=release`) |
| `make release-fast` | Debug-profile CLI inside release bundle path |
| `make clean` | `cargo clean` + UI `dist` / `src-tauri/target` |

Symlinks:

- `macrdp-ui/src-tauri/macrdp-server` → `../../target/debug/macrdp-server` (via `link-cli`)

## Manual build (README path)

```bash
cargo build --release -p macrdp-server
./target/release/macrdp-server
```

UI production:

```bash
cd macrdp-ui && npm ci && npm run tauri -- build
```

Artifacts:

- CLI: `target/release/macrdp-server`
- App: `macrdp-ui/src-tauri/target/release/bundle/macos/macrdp.app`
- DMG: `macrdp-ui/src-tauri/target/release/bundle/dmg/*.dmg`

## GitHub Actions: `.github/workflows/release.yml`

**Triggers**

- Push tags matching `v*`
- `workflow_dispatch` (manual)

**Job `build-macos`** (matrix)

| Runner | Target | Artifacts |
|--------|--------|-----------|
| `macos-latest` | `aarch64-apple-darwin` | DMG + `macrdp-server` binary |
| `macos-15-intel` | `x86_64-apple-darwin` | DMG + `macrdp-server` binary |

**Steps (per matrix leg)**

1. Checkout
2. `dtolnay/rust-action` — stable + cross target
3. `swatinem/rust-cache` — workspace + `macrdp-ui/src-tauri`
4. Node 20 + `npm ci`
5. `cargo build -p macrdp-server --release`
6. `npm run tauri -- build` (`TAURI_SIGNING_PRIVATE_KEY` empty in workflow)
7. Upload DMG and CLI as named artifacts

**Job `release`** (tags only)

- Runs on `ubuntu-latest` after `build-macos`
- Downloads artifacts
- `softprops/action-gh-release@v2` — **draft** release with generated notes
- Attaches both arch DMGs and both `macrdp-server` binaries

### CI gaps (fork opportunities)

- No `cargo test` / `make check` on PRs
- No lint job
- Signing key empty — release DMGs may be unsigned ad hoc
- No automated issue sync with upstream

## Workspace structure

Root `Cargo.toml` workspace members:

- `macrdp-core`, `macrdp-server`, `macrdp-capture`, `macrdp-input`, `macrdp-encode`, `macrdp-audio`, `macrdp-clipboard`

Tauri crate: `macrdp-ui/src-tauri/Cargo.toml` (depends on `macrdp-core`).

Patches:

```toml
[patch.crates-io]
ironrdp-server = { path = "crates/ironrdp-server-gfx" }
ironrdp-acceptor = { path = "crates/ironrdp-acceptor-patched" }
```

## Release checklist (maintainers)

1. Fix/build on target macOS versions locally (`make check`)
2. Bump `macrdp-ui/package.json` / `tauri.conf.json` version if needed
3. Tag `vX.Y.Z` and push
4. Verify draft GitHub release artifacts (arm64 + x86_64)
5. Smoke test: connect with mstsc + one alternate client
6. Publish release notes calling out known [upstream issues](./upstream-issues.md)
