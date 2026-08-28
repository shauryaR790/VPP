# v2.0.0 — Stable automation language

## Highlights

- **Terminal run everywhere** — F5 runs in the integrated terminal (extension 2.0.0); Shift+F5 debugs with `print()` in the Debug Console
- **`std/workflow`** — `sequence`, `parallel`, `retry`, `with_timeout` over typed `Task` steps (uses `process.exec`)
- **CLI parity** — `vpp run`, `vpp.ps1`, and the extension use the same compiler path

## Install

- Windows: `vpp-2.0.0-setup.exe` or portable zip from [Releases](https://github.com/shauryaR790/VPP/releases)
- Dev clone: `.\setup.ps1` (downloads prebuilt 2.0.0; no Rust required)

## Extension

- Marketplace / VSIX: **v++ Language 2.0.0**
- **F5** — run file in terminal
- **Shift+F5** — debug (interpreter DAP)
- **Ctrl+F5** — run without debug (terminal)

## Roadmap

Full automation plan: [AUTOMATION_ROADMAP.md](docs/project/AUTOMATION_ROADMAP.md)
