# v++

A compact typed language for **LLM training and agent tooling**: Python-ish syntax, static types, native binaries — optimized for **speed** and **fewer tokens**.

**License:** MIT · **Version:** 3.0.0 · [Releases](https://github.com/shauryaR790/VPP/releases) · [VS Code extension](https://marketplace.visualstudio.com/items?itemName=vpp-lang.vplusplus) (**26k+ installs**)

## Quick start (Windows)

**Easiest:** download **`vpp-3.0.0-setup.exe`** from [GitHub Releases](https://github.com/shauryaR790/VPP/releases/latest) — no Rust needed.

**From a git clone:**

```powershell
.\setup.ps1          # prebuilt v3.0.0 + extension
.\vpp.ps1 run examples\hello.vpp
.\vpp.ps1 shrink examples\hello.vpp
.\vpp.ps1 tokens examples\hello.vpp
```

Developers: `.\setup.ps1 -Dev` (build from source; needs Rust).

Docs: [website](https://shauryaR790.github.io/VPP/) · [LLM roadmap](docs/project/LLM_ROADMAP.md)

## v3.0 — direction

From v3 onward the product focus is **token-efficient source**, fast interpreter loops, and tooling for corpus prep — not only general automation.

- **`vpp shrink`** — minify `.vpp` for datasets and prompts
- **`vpp tokens`** — per-file lexer-symbol budget
- **`std/llm`** — token estimates in user scripts
- Still: `vpp run`, `vpp build`, LSP, debugger, tests, packages

## Build from source

```powershell
git clone https://github.com/shauryaR790/VPP.git
cd VPP
cargo build --release --features codegen,lsp
cargo test --all-targets
```

See [CONTRIBUTING.md](CONTRIBUTING.md) · [SPEC.md](SPEC.md)

## Links

| | |
|---|---|
| Docs | [shauryaR790.github.io/VPP](https://shauryaR790.github.io/VPP/) |
| Changelog | [CHANGELOG.md](CHANGELOG.md) |
| Roadmap | [docs/project/LLM_ROADMAP.md](docs/project/LLM_ROADMAP.md) |
