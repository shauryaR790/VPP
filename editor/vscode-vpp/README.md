# v++ Language

The official editor for **v++** — a compact typed language built for models, agents, and people who care about **speed** and **small source**.

**vpp-lang** · official extension · [GitHub](https://github.com/shauryaR790/VPP) · [Releases](https://github.com/shauryaR790/VPP/releases/latest)

---

## What this is

`.vpp` files. One toolchain. Run in the terminal, debug with breakpoints, minify for training data, count tokens before you burn context.

v3 is **LLM-first**: less noise in the file, same semantics after `shrink`.

## Get going

1. Grab **vpp-3.0.0-setup.exe** from [Releases](https://github.com/shauryaR790/VPP/releases/latest), or `.\setup.ps1` in a clone.
2. You already found the extension — good.
3. Open a `.vpp` file. **F5** run. **Shift+F5** debug.

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

## v++ CLI (pairs with this extension)

| Command | |
|---------|--|
| `vpp shrink` | Minify in place — corpora, prompts, exports |
| `vpp tokens` | Lexer-symbol estimate |
| `std/llm` | Token helpers in your scripts |
| `vpp run` / `build` / `test` | Interpreter, native binary, project tests |

Roadmap: [LLM_ROADMAP.md](https://github.com/shauryaR790/VPP/blob/main/docs/project/LLM_ROADMAP.md)

## You need

- **vpp** on PATH — installer or `setup.ps1`
- **vppls** for LSP (in the Windows bundle)
- **LLVM** only if you `vpp build` native code

## Links

- [Docs](https://shauryaR790.github.io/VPP/docs.html)
- [Issues](https://github.com/shauryaR790/VPP/issues)
- [MIT](https://github.com/shauryaR790/VPP/blob/main/LICENSE)
