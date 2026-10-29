# Home Assistant Status Updater

A lightweight, cross-platform desktop application built with **Tauri v2**, **Rust**, and **React + TypeScript**. It allows you to monitor and control any Home Assistant `input_select` entity directly from your system tray or via a settings GUI.

---

## ✨ Features

- **System Tray Integration**:
  - Sits unobtrusively in your operating system's system tray / menu bar.
  - Clicking the tray icon displays a menu with the current entity state and all available options.
  - Selecting any option immediately calls the Home Assistant REST API and updates the menu checkmark.
  - Quick access to "Refresh Now", "Settings...", and "Exit".
- **Cross-Platform**:
  - Works on Windows, macOS, and Linux powered by Tauri v2.
  - Minimizes to tray on close to maintain background connectivity.
- **Rich Configuration GUI**:
  - **Home Assistant Base URL**: Enter your local IP, hostname (`http://homeassistant.local:8123`), Nabu Casa, or custom reverse-proxy URL.
  - **Entity ID**: Specify any `input_select.*` entity (e.g., `input_select.house_mode`).
  - **Auto-Discovery**: Click **"Detect Entities"** to discover and choose from all `input_select` entities on your Home Assistant instance.
  - **Long-Lived Access Token**: Securely enter your token (with show/hide toggle). Can also import automatically from a local `credentials.json` if available.
  - **Configurable Polling Interval**: Set automatic background sync (every 15s, 30s, 1m, 2m, 5m, or manual only) so tray status always reflects changes made from other devices.
  - **Test Connection**: Verify API connectivity, entity existence, and option list before saving.
  - **Interactive State Picker**: Switch state directly inside the GUI or from the tray.

---

## 🚀 Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install) (stable)

### Running in Development

```bash
# Install frontend dependencies
npm install

# Start Vite dev server and Tauri application
npm run tauri dev
```

### Running Tests

Run frontend tests (Vitest + Testing Library):
```bash
npm test
```

Run backend tests (Cargo test with in-memory HTTP server):
```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### Building for Production

To create a release executable and installer for your OS:

```bash
npm run tauri build
```

The resulting binaries will be placed in `src-tauri/target/release/`.

---

## ⚙️ Home Assistant REST API Endpoints Used

The application communicates with Home Assistant through standard REST API endpoints:

1. **Get Entity State & Options**:
   - `GET /api/states/<entity_id>`
   - Retrieves the current state, `friendly_name`, and available `options` array from entity attributes.
2. **Select Option**:
   - `POST /api/services/input_select/select_option`
   - Payload:
     ```json
     {
       "entity_id": "input_select.house_mode",
       "option": "Away"
     }
     ```
3. **Discover Entities**:
   - `GET /api/states`
   - Filters entities with the prefix `input_select.`.

---

## 🔒 Configuration & Credentials

Configuration is persistently saved in your system's application config folder:
- **Windows**: `%APPDATA%\com.statusupdater.desktop\config.json`
- **macOS**: `~/Library/Application Support/com.statusupdater.desktop/config.json`
- **Linux**: `~/.config/com.statusupdater.desktop/config.json`

If a `credentials.json` file is present in the application directory with the format:
```json
{
  "ha-access-token": "YOUR_LONG_LIVED_ACCESS_TOKEN"
}
```
the application will detect and allow quick importing of the token.
