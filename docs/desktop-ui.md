# Desktop UI (Tauri)

## Stack

- **Tauri 2** (`macrdp-ui/src-tauri`)
- **React 18** + **Vite** + **TypeScript**
- **React Router** — main window routes + separate popover window
- **Radix UI / Base UI** — controls
- **SQLite** (`database.rs`) — connection history and traffic statistics

Product id: `com.macrdp.app` (bundle id in Makefile: same).

## Windows

| Label | Route / URL | Role |
|-------|-------------|------|
| `main` | `/`, `/settings`, … | Primary console |
| `popover` | `/popover` | Small tray-adjacent panel |

Tray icon configured in `tauri.conf.json` (`trayIcon.id = main-tray`).

`Info.plist` sets `LSUIElement=true` (agent / menu-bar style app). This interacts badly with tray on **macOS 26** — see [upstream-issues.md](./upstream-issues.md).

## Pages

| Path | Component | Purpose |
|------|-----------|---------|
| `/` | `Dashboard` | Status bar, FPS chart, metrics, connections |
| `/settings` | `Settings` | Split panel; categories; `set_config` |
| `/permissions` | `Permissions` | TCC guidance; opened on first launch if server running and perms missing |
| `/logs` | `Logs` | Live logs from `LogBridgeLayer` |
| `/statistics` | `Statistics` | Historical charts from DB |
| `/about` | `About` | Version / credits |
| `/popover` | `Popover` | Compact status (tray) |

## Tauri commands (IPC)

Frontend wraps these in `src/lib/ipc.ts` as `api.*`.

| Command | Description |
|---------|-------------|
| `start_server` | `macrdp_core::start_server` in-process |
| `stop_server` | Graceful shutdown via `ServerHandle` |
| `get_server_status` | Running, state string, uptime |
| `get_metrics` | FPS, bitrate, RTT, encode ms, pending ACKs, … |
| `get_connections` | Active RDP sessions |
| `get_permissions` | Screen capture + accessibility flags |
| `get_config` / `set_config` | UI config ↔ server; `restart_required` flag |
| `get_logs` | Buffered log entries |
| `get_connection_history` | SQLite history |
| `get_traffic_stats` | Aggregated traffic for statistics page |
| `check_for_updates` | Update check hook |
| `show_main_window` | Focus main window from tray |
| `open_system_preferences` | Deep link to privacy panes |
| `quit_app` | Stop server and exit |
| `set_autostart` | Autostart plugin |

## Events (server → UI)

`TauriEventBridge` implements `ServerEventHandler` and emits Tauri events (e.g. `server-status`, metrics, connections, logs). Hooks under `src/hooks/` subscribe for dashboard and charts.

## Dev workflow

```bash
make dev-ui    # cargo CLI + tauri dev
make dev-front # vite only
```

Dev URL: `http://localhost:1420` (`tauri.conf.json` `devUrl`).

## Bundled CLI

Release builds embed or sidecar the `macrdp-server` binary depending on `MACRDP_CLI_PROFILE` and Tauri resource config. `link-cli` keeps debug iteration paths consistent during development.
