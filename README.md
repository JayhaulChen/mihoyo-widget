# mihoyo-widget

Desktop widget for Honkai: Star Rail game data — stamina, challenges, banners, battle events, and more.

Built with **Tauri v2** (Rust backend) + **Vite** (vanilla JS frontend).
Multi-platform, with workspace layout ready for future game support
(Genshin, ZZZ).

## Features

- **仪表盘** — Real-time stamina ring, 2×3 status grid (sign-in,
  expeditions, weekly bosses, training)
- **挑战** — Forgotten Hall, Pure Fiction, Apocalyptic Shadow,
  Challenge Peak + weekly progress bars
- **活动·档案** — Monthly stellar jade ledger, active card pools,
  limited-time events with countdown and progress bars, Simulated
  Universe archives (Nous/Magic/Locust)
- **System tray** — Always-on notification area icon, right-click menu
  (show/hide, refresh, quit). Toggle between **window mode** and
  **notification-only mode** (background polling + system notifications,
  no UI window)
- **Theme toggle** — Dark/Light mode, persisted to localStorage
- **Desktop notifications** — Configurable rules: stamina threshold
  sliders, expedition completion, sign-in reminders, weekly rogue score
  reminders, daily digest. Each rule individually togglable with custom
  time settings
- **Onboarding wizard** — 5-step first-run guide: welcome → data
  directory picker → in-app WebView login → feature tour → ready.
  Reopenable from tray menu anytime
- **In-app WebView login** — Embedded miHoYo login page, auto-captures
  cookie/SToken on successful login, fills into settings automatically
- **Settings redesign** — iOS-style grouped menu with drill-down
  subpages, auto-save on toggle/blur/navigate-back, no manual save
  button

## Project Structure

```text
├── apps/
│   └── desktop/         Tauri v2 desktop app (binary, tray, window)
├── packages/
│   ├── core/            Shared Rust: HTTP client, DS signing, config,
│   │                    KV cache
│   ├── game-hsr/        HSR-specific: API client, data types, typed
│   │                    cache, notify rules
│   └── frontend/        Frontend (Vite, JS + CSS, HTML)
├── Cargo.toml           Workspace root
├── package.json         Workspace forwarding scripts
└── LICENSE (MIT)
```

## Getting Started

### Prerequisites

| Platform | System Dependencies |
| --- | --- |
| **Linux** | `sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libssl-dev` |
| **macOS** | Xcode Command Line Tools (`xcode-select --install`) |
| **Windows** | Visual Studio Build Tools (or Visual Studio Community with "Desktop development with C++") |

### Run in development mode

```bash
git clone https://github.com/<your-org>/mihoyo-widget.git
cd mihoyo-widget

# Frontend dependencies
cd packages/frontend && npm install && cd ../..

# Launch (builds Rust + frontend with hot reload)
cargo tauri dev
```

### Build for production

```bash
cd packages/frontend && npm install && npm run build && cd ../..
cd apps/desktop && cargo tauri build
```

**First run**: If no credentials are found, the app opens an onboarding
wizard:

1. Welcome screen
2. Pick a data directory (or use default)
3. Log in via embedded WebView (auto-captures your Cookie/SToken)
4. Feature introduction
5. Dashboard loads with your data

You can re-open the wizard anytime from system tray menu → 「欢迎引导」.

## Configuration

There are three ways to configure the app:

1. **Via the onboarding wizard** (first run) — In-app WebView login,
   auto-fills credentials.
2. **Via the settings GUI** — Right-click anywhere or click the gear
   icon ⚙, navigate to the settings panel. Credentials and all
   preferences auto-save on change.
3. **Via config file** — Place a `Mihoyo-env.json` in your **Downloads** directory or OS config directory:

```json
{
  "cookie": "your_full_mihoyo_cookie",
  "stoken": "your_stoken",
  "uid": "your_game_uid",
  "stuid": "your_account_stuid",
  "mid": "your_mid"
}
```

Runtime edits are saved to the OS-appropriate config directory:

- **Linux**: `~/.config/mihoyo-widget/runtime.json`
- **macOS**: `~/Library/Application Support/mihoyo-widget/runtime.json`
- **Windows**: `%APPDATA%/mihoyo-widget/runtime.json`

You can also set all values via environment variables:
`MIHOYO_COOKIE`, `MIHOYO_STOKEN`, `MIHOYO_UID`, etc.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## Platform Support

| Platform | Status | Notes |
| --- | --- | --- |
| Linux (x86_64) | ✅ | Tested, full support |
| Windows (x86_64) | ✅ | CI-verified |
| macOS (Apple Silicon) | ✅ | CI-verified, native ARM build |
| macOS (Intel) | ⚠️ | Build on request via `cargo tauri build` |

## Tech Stack

| Layer | Technology |
| --- | --- |
| Desktop shell | Tauri v2 (Rust), tray-icon, image-png |
| Backend | tokio async, reqwest, rusqlite (SQLite WAL) |
| Frontend | Vanilla JS, CSS custom properties (theming) |
| Auth | DS2 signing (X4 salt), device-fp registration |

## License

MIT
