# v++ Language

Official VS Code support for [v++](https://github.com/shauryaR790/VPP): Python-ish syntax, static types, native binaries.

Compiler **v1.0.4** on [GitHub Releases](https://github.com/shauryaR790/VPP/releases/latest). Extension **1.2.2** on Marketplace.

## Quick start

1. Install `vpp-1.0.4-setup.exe` from GitHub Releases (Windows).
2. Install this extension (publisher: **vpp-lang**).
3. Open a `.vpp` file. **F5** to debug, **Ctrl+F5** to run.

```powershell
vpp run examples\hello.vpp
vpp debug examples\hello.vpp
vpp test
```

## Features

| Feature | What you get |
|---------|----------------|
| Debug (F5) | Breakpoints, step, locals via `vpp debug --dap` |
| Run (Ctrl+F5) | `vpp run` on the active file |
| Watch | Re-run on save (`vpp watch`) |
| Test Explorer | Sidebar tests from `test "..."` blocks |
| Format on save | `vpp fmt` (needs `vpp` 1.0.1+) |
| LSP | Diagnostics, completion, go-to-definition (`vppls`) |

## Requirements

- **vpp** on PATH (run, debug, fmt, test)
- **vppls** for LSP (bundled in the Windows installer)
- **LLVM** only for `vpp build` native codegen

## Links

- [Docs](https://github.com/shauryaR790/VPP/tree/main/docs)
- [Issues](https://github.com/shauryaR790/VPP/issues)
- [MIT License](https://github.com/shauryaR790/VPP/blob/main/LICENSE)
