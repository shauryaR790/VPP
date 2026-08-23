# v++ Language  -  official VS Code extension

**Write it simply. Compile it natively.**

Official language support for **[v++](https://github.com/shauryaR790/VPP)**  -  a compiled language that reads like Python but ships native binaries.

> **v1.0.4** (compiler) · extension **1.2.1** (Marketplace)

> **Parity Promise:** the same `.vpp` file runs in **REPL**, **watch**, **debug (F5)**, and **`vpp build`**.

**Official extension:** publisher **`vpp-lang`** · ID **`vpp-lang.vplusplus`**

---

## Quick start

1. **Compiler**  -  [GitHub Releases v1.0.4](https://github.com/shauryaR790/VPP/releases/latest) (`vpp-1.0.4-setup.exe` on Windows).
2. **Extension**  -  install **v++ Language** by **vpp-lang**.
3. **Debug**  -  open a `.vpp` file, press **F5**. **Ctrl+F5** runs without breakpoints.

```powershell
vpp run examples\hello.vpp
vpp debug examples\hello.vpp
vpp watch examples\hello.vpp
vpp repl
vpp test
vpp search hello
```

---

## Features (v1.0.4)

| Feature | What you get |
|---------|----------------|
| **Debug (F5)** | Breakpoints, step, next, locals  -  same interpreter as `vpp run` |
| **Run (Ctrl+F5)** | `vpp run` on active file |
| **Watch** | Live re-run on save (`vpp watch`) |
| **REPL** | Interactive session in terminal |
| **Benchmark** | `vpp bench` timing |
| **Test Explorer** | Sidebar tests from `test "..."` blocks |
| **Format on save** | `vpp fmt`  -  fixed in 1.0.4 (requires `vpp` ≥ 1.0.1) |
| **LSP** | Diagnostics, completion, go-to-definition (`vppls`) |
| **Registry search** | `vpp search` from command palette |
| **Snippets + icons** | Official V++ wordmark |

**Download compiler:** [GitHub Releases v1.0.4](https://github.com/shauryaR790/VPP/releases/latest) (Windows installer + portable zip).

---

## Commands

`Ctrl+Shift+P` → `v++`:

| Command | Shortcut |
|---------|----------|
| **Debug File** | F5 |
| **Run File** | Ctrl+F5, Ctrl+Shift+R |
| **Watch File** | toolbar eye icon |
| **Format Document** | Shift+Alt+F |
| **Refresh Test Explorer** |  -  |
| **Search Package Registry** |  -  |

---

## Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `vpp.compilerPath` | *(auto)* | Path to `vpp` |
| `vpp.languageServerPath` | `vppls` | Language server |
| `vpp.enableLanguageServer` | `true` | IntelliSense |
| `vpp.formatOnSave` | `true` | Format `.vpp` on save |

---

## Requirements

- **vpp**  -  run, debug, check, fmt, test, watch, bench
- **vppls**  -  LSP (bundled in release or `cargo build --features lsp --bin vppls`)
- **LLVM**  -  only for `vpp build` native codegen

---

## Docs

- [Documentation](https://github.com/shauryaR790/V-/tree/main/docs)
- [Parity Promise](https://github.com/shauryaR790/V-/blob/main/docs/project/PARITY_PROMISE.md)
- [Report issues](https://github.com/shauryaR790/V-/issues)

MIT  -  see [LICENSE](https://github.com/shauryaR790/V-/blob/main/LICENSE).
