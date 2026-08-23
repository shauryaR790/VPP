# VS Code setup for v++

## Install

1. Install the compiler ([install guide](install.md)).
2. Install extension **v++ Language** (publisher: **vpp-lang**).

## Run code

1. Open a folder with `.vpp` files.
2. Open a file  -  bottom-right should say **v++**.
3. Press **F5** or **Ctrl+Shift+R** to run the active file.

## Language server (IntelliSense)

Build `vppls` once (or use a release that includes it):

```powershell
cargo build --release --features lsp --bin vppls
```

Settings (`File → Preferences → Settings`, search `vpp`):

| Setting | Purpose |
|---------|---------|
| `vpp.compilerPath` | Path to `vpp.exe` |
| `vpp.languageServerPath` | Path to `vppls` |
| `vpp.enableLanguageServer` | Red squiggles, completion, go-to-definition |

## Commands

| Command | Shortcut |
|---------|----------|
| v++: Run File | F5, Ctrl+Shift+R |
| v++: Check File | Command Palette |
| v++: Run Tests | Command Palette |

See also: [Language server](../guides/language-server.md), [Troubleshooting](../guides/troubleshooting.md).
