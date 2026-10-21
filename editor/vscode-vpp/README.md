# v++ Language

Official VS Code support for [v++](https://github.com/shauryaR790/VPP): a compact, typed language tuned for **LLM training and tool use** — fast runs, readable syntax, **minimal tokens**.

**Extension 3.0.0** · **Compiler v3.0.0** · [Marketplace](https://marketplace.visualstudio.com/items?itemName=vpp-lang.vplusplus) · [Releases](https://github.com/shauryaR790/VPP/releases/latest) · **26k+ installs**

## Quick start

1. Install **vpp-3.0.0-setup.exe** from [GitHub Releases](https://github.com/shauryaR790/VPP/releases/latest) (Windows), or run `.\setup.ps1` from a clone.
2. Install this extension — search **v++ Language** (publisher **vpp-lang**), or install `vplusplus-3.0.0.vsix` from Releases.
3. Open a `.vpp` file. **F5** to run; **Shift+F5** to debug.

```powershell
vpp run examples\hello.vpp
vpp shrink examples\hello.vpp    # minify for corpora / prompts
vpp tokens examples\hello.vpp    # budget lexer symbols
vpp debug examples\hello.vpp
vpp test
```

## v3.0 — LLM-first tooling

| Command | What |
|---------|------|
| `vpp shrink` | Strip layout noise; smaller files for training data |
| `vpp tokens` | Lexer-symbol estimate per file |
| `std/llm` | `est_text_tokens`, `est_bytes_tokens` in scripts |

Roadmap: [LLM_ROADMAP.md](https://github.com/shauryaR790/VPP/blob/main/docs/project/LLM_ROADMAP.md)

## Editor features

| Feature | Shortcut | What you get |
|---------|----------|----------------|
| Run | **F5** / Ctrl+F5 | `vpp run` in the integrated terminal |
| Debug | **Shift+F5** | Breakpoints, step, locals; `print()` in Debug Console |
| Watch | command | Re-run on save (`vpp watch`) |
| Test Explorer | sidebar | Tests from `test "..."` blocks |
| Format on save | — | `vpp fmt` |
| LSP | auto | Diagnostics, completion, go-to-definition (`vppls`) |

## Requirements

- **vpp** on PATH (run, shrink, tokens, debug, fmt, test) — installer or `setup.ps1`
- **vppls** for LSP (bundled in the Windows installer)
- **LLVM** only for `vpp build` native codegen

## Links

- [Docs site](https://shauryaR790.github.io/VPP/docs.html)
- [Issues](https://github.com/shauryaR790/VPP/issues)
- [MIT License](https://github.com/shauryaR790/VPP/blob/main/LICENSE)
