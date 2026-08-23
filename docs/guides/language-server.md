# Language server (vppls)

`vppls` provides:

- **Diagnostics**  -  red squiggles from the type checker
- **Completion**  -  keyword and context suggestions
- **Go to definition**  -  jump to symbols (best-effort)

## Build

```powershell
cargo build --release --features lsp --bin vppls
```

Release bundles include `vppls.exe`.

## VS Code

Extension setting `vpp.languageServerPath` defaults to `vppls` on PATH.

Disable with `vpp.enableLanguageServer: false`.

## Protocol

Standard LSP over stdio. Started automatically by the VS Code extension when a `.vpp` file is open.
