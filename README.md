# v++

**v++** — compact, typed, built for **LLMs and shipping**. Readable `.vpp` source, fewer tokens, fast `vpp run`, native output when you `build`.

**MIT** · **v3.0.0** · [Releases](https://github.com/shauryaR790/VPP/releases) · [Extension](https://marketplace.visualstudio.com/items?itemName=vpp-lang.vplusplus) (**26k+ installs**)

## Start (Windows)

Download **`vpp-3.0.0-setup.exe`** from [Releases](https://github.com/shauryaR790/VPP/releases/latest). No toolchain drama.

Clone path:

```powershell
.\setup.ps1
.\vpp.ps1 run examples\hello.vpp
.\vpp.ps1 shrink examples\hello.vpp
.\vpp.ps1 tokens examples\hello.vpp
```

Hacking the compiler itself: `.\setup.ps1 -Dev` (needs a local build setup).

[Docs](https://shauryaR790.github.io/VPP/) · [LLM roadmap](docs/project/LLM_ROADMAP.md)

## v3 direction

Token-efficient source. Fast loops. Tooling for datasets.

- **`vpp shrink`** — minify `.vpp` for training / prompts
- **`vpp tokens`** — budget lexer symbols
- **`std/llm`** — estimates in scripts
- Plus: run, build, debug, LSP, packages, tests

## From source

```powershell
git clone https://github.com/shauryaR790/VPP.git
cd VPP
cargo build --release --features codegen,lsp
cargo test --all-targets
```

[CONTRIBUTING.md](CONTRIBUTING.md) · [SPEC.md](SPEC.md) · [CHANGELOG.md](CHANGELOG.md)
