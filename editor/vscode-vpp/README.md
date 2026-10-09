# v++ Language

Official VS Code support for [v++](https://github.com/shauryaR790/VPP): Python-ish syntax, static types, native binaries.

**Extension 2.0.0** · **Compiler v2.0.0** · [Marketplace](https://marketplace.visualstudio.com/items?itemName=vpp-lang.vplusplus) · [Releases](https://github.com/shauryaR790/VPP/releases/latest)

## Quick start

1. Install **vpp-2.0.0-setup.exe** from [GitHub Releases](https://github.com/shauryaR790/VPP/releases/latest) (Windows), or run `.\setup.ps1` from a clone.
2. Install this extension — search **v++ Language** on the Marketplace (publisher **vpp-lang**), or install `vplusplus-2.0.0.vsix` from Releases.
3. Open a `.vpp` file. **F5** to run in the terminal; **Shift+F5** to debug.

```powershell
vpp run examples\hello.vpp
vpp run examples\workflow_smoke.vpp
vpp debug examples\hello.vpp
vpp test
```

## Features

| Feature | Shortcut | What you get |
|---------|----------|----------------|
| Run | **F5** / Ctrl+F5 | `vpp run` in the integrated terminal |
| Debug | **Shift+F5** | Breakpoints, step, locals; `print()` in Debug Console |
| Watch | command | Re-run on save (`vpp watch`) |
| Test Explorer | sidebar | Tests from `test "..."` blocks |
| Format on save | — | `vpp fmt` |
| LSP | auto | Diagnostics, completion, go-to-definition (`vppls`) |

## v2.0.0 workflows

Works with compiler **v2.0.0** stdlib (interpreter; native workflow parity in progress):

- `std.workflow` — `sequence`, `parallel`, `retry`, `with_timeout`
- `std.process.exec` — typed process I/O
- `std.env`, `std.dir`, `std.logging`

See [AUTOMATION_ROADMAP.md](https://github.com/shauryaR790/VPP/blob/main/docs/project/AUTOMATION_ROADMAP.md).

## Requirements

- **vpp** on PATH (run, debug, fmt, test) — installer or `setup.ps1`
- **vppls** for LSP (bundled in the Windows installer)
- **LLVM** only for `vpp build` native codegen

## Links

- [Docs site](https://shauryaR790.github.io/VPP/docs.html)
- [Issues](https://github.com/shauryaR790/VPP/issues)
- [MIT License](https://github.com/shauryaR790/VPP/blob/main/LICENSE)
