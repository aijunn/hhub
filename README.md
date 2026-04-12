# HHub

<p align="center">
  <img src="./src/assets/hhub-brand.svg" alt="HHub icon" width="96" height="96" />
</p>

<p align="center">
  A polished desktop video library for local media management.<br />
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-2.x-111827?style=flat-square" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Vue-3-0f172a?style=flat-square" alt="Vue 3" />
  <img src="https://img.shields.io/badge/Rust-Backend-1f2937?style=flat-square" alt="Rust backend" />
  <img src="https://img.shields.io/badge/Platform-Desktop-374151?style=flat-square" alt="Desktop app" />
</p>

## Overview

HHub is a local-first video manager focused on a clean desktop workflow: import videos, organize them with tags, preview and play them inside the app, and keep sensitive content behind an optional app lock.

## Screenshots

![Library overview](./docs/screenshots/library-overview.png)

## Features

- Import local videos from the file picker, drag and drop, or clipboard paste.
- Organize videos with favorites, rename, delete, export, and tag management.
- Filter the library by favorites, tags, and search keywords.
- Play videos inside the app with seek, volume, rate control, theater mode, and fullscreen.
- Use delayed delete with undo to reduce accidental removals.
- Protect the app with an optional password lock.
- Pause or lock automatically when the window loses focus.
- Enjoy a compact macOS-inspired interface with a custom title bar and collapsible sidebar.

## How to use

```bash
pnpm install
pnpm tauri build
```
