# v++ Language

Official **vpp-lang** editor support for the v++ programming language.

Compact typed `.vpp` source · **shrink** & **tokens** for LLM workflows · **run** · **debug** · **LSP**

[GitHub](https://github.com/shauryaR790/VPP) · [Releases](https://github.com/shauryaR790/VPP/releases/latest) · [Docs](https://shauryaR790.github.io/VPP/docs.html)

---

## What this is

One toolchain for `.vpp` files: terminal run, breakpoints, minify for training data, token estimates before you burn context.

v3 is **LLM-first** — less noise in the file, same semantics after `shrink`.

## Get going

1. **vpp-3.0.0-setup.exe** from [Releases](https://github.com/shauryaR790/VPP/releases/latest), or `.\setup.ps1` in a clone.
2. Open a `.vpp` file. **F5** run. **Shift+F5** debug.

```powershell
vpp run examples\hello.vpp
vpp shrink examples\hello.vpp
vpp tokens examples\hello.vpp
```

## In the editor

| | |
|--|--|
| **F5** / Ctrl+F5 | Run in integrated terminal |
| **Shift+F5** | Debug — breakpoints, step, locals, `print()` in Debug Console |
| Command palette | **Shrink for LLM**, **Token Count**, Watch, REPL, tests |
| Save | Optional format via `vpp fmt` |
| Open a `.vpp` | LSP diagnostics, completion, go-to-def |

## v++ CLI

| Command | |
|---------|--|
| `vpp shrink` | Minify in place — corpora, prompts, exports |
| `vpp tokens` | Lexer-symbol estimate |
| `std/llm` | Token helpers in scripts |
| `vpp run` / `build` / `test` | Interpreter, native binary, project tests |

[Roadmap](https://github.com/shauryaR790/VPP/blob/main/docs/project/LLM_ROADMAP.md)

## Requirements

- **vpp** on PATH — installer or `setup.ps1`
- **vppls** for LSP (Windows bundle)
- **LLVM** only for `vpp build`

## Links

- [Issues](https://github.com/shauryaR790/VPP/issues)
- [MIT License](https://github.com/shauryaR790/VPP/blob/main/LICENSE)
