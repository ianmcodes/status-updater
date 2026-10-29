# Repository Guidelines

## Project Structure & Module Organization

This is a Tauri v2 desktop application for updating Home Assistant `input_select` entities. The React/TypeScript UI lives in `src/`: `App.tsx` owns the settings and state UI, `types.ts` holds shared frontend types, and `icons.tsx` contains UI icons. Place frontend tests in `src/test/` (for example, `App.test.tsx`). Static web assets belong in `public/`; Tauri icons live in `src-tauri/icons/`.

The Rust backend is in `src-tauri/src/`. Keep Home Assistant HTTP behavior in `ha_client.rs`, persistent settings in `config.rs`, shared backend models in `models.rs`, and tray behavior in `tray.rs`. Tauri configuration and permissions are in `src-tauri/tauri.conf.json` and `src-tauri/capabilities/`.

## Build, Test, and Development Commands

- `npm install` installs frontend dependencies.
- `npm run tauri dev` starts Vite and launches the desktop app for development.
- `npm run build` type-checks TypeScript and creates the frontend bundle in `dist/`.
- `npm test` runs the Vitest frontend suite once; use `npm run test:watch` while editing UI code.
- `cargo test --manifest-path src-tauri/Cargo.toml` runs Rust backend tests.
- `npm run tauri build` creates production desktop bundles.

## Coding Style & Naming Conventions

Follow the existing code style: two-space indentation, double quotes, semicolons, and trailing commas in TypeScript. Use PascalCase for React components and types, camelCase for functions and variables, and descriptive test names such as `"selects an option and updates state"`. Follow standard Rust formatting and naming: `snake_case` functions/modules and `PascalCase` types. Run `cargo fmt --check --manifest-path src-tauri/Cargo.toml` before submitting Rust changes. No frontend formatter or linter is currently configured; preserve the surrounding style.

## Testing Guidelines

Use Vitest with Testing Library for UI behavior. Mock Tauri APIs in frontend tests and assert observable text, controls, and invoked commands. Add Rust tests alongside the module they cover, especially for API requests, configuration persistence, and error paths. Run both test commands when a change crosses the frontend/backend boundary. There is no stated coverage threshold; meaningful regression coverage is expected for changed behavior.

## Commit & Pull Request Guidelines

Use brief, imperative commit subjects consistent with history, e.g. `Add tests` or `Build release workflow`. Keep commits focused. Pull requests should explain the behavior change, link the relevant issue when applicable, list the tests run, and include screenshots for visible UI changes. Do not commit `credentials.json`, `.env` files, tokens, or generated build output.
