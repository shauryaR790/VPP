v1.0.5 — Automation foundation
================================

## Automation

- **Typed process execution** — `process.exec(program, args, opts) -> Result<ProcessOutput, string>` with stdout/stderr capture
- **Environment** — `std/env.vpp` (`get`, `set`)
- **Directories** — `std/dir.vpp` (`list`, `exists`, `create`)
- **Structured logging** — `std/logging.vpp` (`info`, `warn`, `log_error`)
- **C runtime** — `runtime/vpp_automation.c` (native builds)
- **Example** — `examples/automation_smoke.vpp`
- **Roadmap** — [AUTOMATION_ROADMAP.md](docs/project/AUTOMATION_ROADMAP.md) (v1.0.5 → v2.0.0)

## Tooling

- **VS Code extension 1.0.5** — `vplusplus-1.0.5.vsix` attached (upload to Marketplace separately if needed)
- Debug (F5), Test Explorer, watch, REPL, LSP, formatter fix
- CMake modules remain from v1.0.4 in the installer

## Install

- Windows: `vpp-1.0.5-setup.exe` or portable zip
- VS Code: install `vplusplus-1.0.5.vsix` from this release, or Marketplace when published

## Docs

- https://shauryaR790.github.io/VPP/docs.html
