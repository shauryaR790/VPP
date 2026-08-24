# v++ Language

Official VS Code support for [v++](https://github.com/shauryaR790/VPP): Python-ish syntax, static types, native binaries.

**Extension 1.2.3** · **Compiler v1.0.5** · [GitHub Releases](https://github.com/shauryaR790/VPP/releases/latest)

## Quick start

1. Install `vpp-1.0.5-setup.exe` from GitHub Releases (Windows).
2. Install this extension (publisher: **vpp-lang**) — Marketplace or `vplusplus-1.2.3.vsix` from Releases.
3. Open a `.vpp` file. **F5** to debug, **Ctrl+F5** to run.

```powershell
vpp run examples\hello.vpp
vpp run examples\automation_smoke.vpp
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

## Compiler v1.0.5 automation APIs

Works with the v1.0.5 stdlib (interpreter + native parity):

- `std.process.exec(program, args, opts)` → `Result<ProcessOutput, string>`
- `std.env.get` / `std.env.set`
- `std.dir.list` / `exists` / `create`
- `std.logging.info` / `warn` / `log_error`

See [AUTOMATION_ROADMAP.md](https://github.com/shauryaR790/VPP/blob/main/docs/project/AUTOMATION_ROADMAP.md).

## Requirements

- **vpp** on PATH (run, debug, fmt, test)
- **vppls** for LSP (bundled in the Windows installer)
- **LLVM** only for `vpp build` native codegen

## Links

- [Docs site](https://shauryaR790.github.io/VPP/docs.html)
- [Issues](https://github.com/shauryaR790/VPP/issues)
- [MIT License](https://github.com/shauryaR790/VPP/blob/main/LICENSE)
