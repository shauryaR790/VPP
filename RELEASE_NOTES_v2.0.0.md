# v2.0.0 — Stable automation language

## Highlights

- **Terminal run everywhere** — F5 runs in the integrated terminal; Shift+F5 debugs; `print()` in Debug Console
- **`std/workflow`** — `sequence`, `parallel`, `retry`, `with_timeout` over typed `Task` steps
- **CLI parity** — `vpp run`, `vpp.ps1`, and the extension use the same compiler

## Install (Windows)

- **Compiler:** `vpp-2.0.0-setup.exe` or `vpp-v2.0.0-windows-x64.zip` (below)
- **Dev clone:** `.\setup.ps1` (downloads prebuilt 2.0.0; no Rust required)

## VS Code extension 2.0.0

**[Live on Marketplace](https://marketplace.visualstudio.com/items?itemName=vpp-lang.vplusplus)** (publisher: `vpp-lang`)

| Key | Action |
|-----|--------|
| **F5** | Run in terminal |
| **Shift+F5** | Debug |
| **Ctrl+F5** | Run without debug |

Also attached: **`vplusplus-2.0.0.vsix`** (same build as Marketplace).

## Assets

- `vpp-2.0.0-setup.exe` — Windows installer (when CI completes)
- `vpp-v2.0.0-windows-x64.zip` — portable bundle
- `vplusplus-2.0.0.vsix` — VS Code / Cursor extension

## Roadmap

[AUTOMATION_ROADMAP.md](docs/project/AUTOMATION_ROADMAP.md)
