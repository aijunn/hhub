# Repository Guidelines

## Project Structure & Module Organization
This repository is a Tauri desktop app for local video management.

- `src/`: Vue 3 frontend. Main UI lives in `App.vue`; shared API helpers are in `src/lib/`; Pinia state is in `src/stores/`; app-wide styles are in `src/styles.css`; static assets and icons live in `src/assets/`.
- `src-tauri/`: Rust backend and Tauri config. Core media-library logic is in `src-tauri/src/media_library.rs`; command wiring is in `src-tauri/src/lib.rs`; window and bundle settings are in `src-tauri/tauri.conf.json`.
- `src/lib/*.test.ts`: Vitest unit tests for frontend-side pure helpers.
- `src-tauri/icons/`: Generated app icons used by Tauri bundles.

## Build, Test, and Development Commands
- `pnpm dev`: Run the Vite frontend in development mode.
- `pnpm tauri dev`: Run the full desktop app locally.
- `pnpm build`: Type-check Vue code and build the frontend bundle.
- `pnpm exec vitest run`: Run frontend helper tests.
- `cargo test --manifest-path src-tauri/Cargo.toml`: Run Rust unit tests.
- `pnpm tauri build --debug --no-bundle`: Build the desktop app without packaging installers.

## Coding Style & Naming Conventions
- Use TypeScript and Rust idioms already present in the repo.
- Prefer 2-space indentation in Vue/TS/CSS and standard Rust formatting in `src-tauri/`.
- Use `camelCase` for TS variables/functions, `PascalCase` for Vue component types, and `snake_case` for Rust functions/fields exposed to Tauri when appropriate.
- Keep UI state mutations in `App.vue` or `src/stores/library.ts`; keep pure reusable logic in `src/lib/`.
- Use `apply_patch` for manual file edits; avoid rewriting whole files unnecessarily.

## Testing Guidelines
- Add Vitest tests for new frontend pure helpers in `src/lib/` using `*.test.ts`.
- Add Rust unit tests near the implementation in `src-tauri/src/media_library.rs`.
- Prefer test-first changes for bug fixes and state helpers.
- Before finishing work, run at least `pnpm build` and the most relevant Rust or Vitest test command.

## Commit & Pull Request Guidelines
- Follow the existing commit style: Conventional Commits such as `feat: ...` and `fix: ...`.
- Keep commits focused by subsystem or behavior.
- PRs should include: a concise summary, testing performed, screenshots/GIFs for UI changes, and any migration or config notes if Tauri/Rust settings changed.

## Security & Configuration Tips
- Treat `src-tauri/capabilities/default.json` and `src-tauri/tauri.conf.json` as sensitive integration points; review permission changes carefully.
- Changes to `app_settings` or SQLite schema in `media_library.rs` should include migration-safe logic and a regression test for older local databases.
