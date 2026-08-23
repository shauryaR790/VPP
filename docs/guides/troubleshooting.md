# Troubleshooting

## `vpp` not recognized

- Reopen terminal after install
- Add install dir to PATH (see [install guide](../getting-started/install.md))
- Or set `vpp.compilerPath` in VS Code

## SmartScreen / security warning

Open-source installers may trigger a one-time Windows SmartScreen prompt. Choose **More info → Run anyway**, or verify the download from [GitHub Releases](https://github.com/shauryaR790/V-/releases).

## `vpp build` fails  -  clang not found

- Use release installer (bundled clang), or
- `winget install LLVM.LLVM` and set `LLVM_SYS_221_PREFIX`

## No syntax highlighting

- Install extension **v++ Language** (vpp-lang)
- Reload window; confirm language mode is **v++**

## LSP not working

- Build `vppls`: `cargo build --features lsp --bin vppls`
- Check Output panel → v++ Language Server

## Native vs interpreter mismatch

Run parity check:

```powershell
.\stress.ps1
```

File issues: [GitHub Issues](https://github.com/shauryaR790/V-/issues).
