# v++

A small compiled language: Python-ish syntax, static types, native binaries via LLVM.

**License:** MIT · **Version:** 1.0.4 · [Releases](https://github.com/shauryaR790/VPP/releases)

## Quick start (Windows)

```powershell
# 1. Install vpp-1.0.4-setup.exe from GitHub Releases
# 2. Install "v++ Language" in VS Code (publisher: vpp-lang)
vpp run examples\hello.vpp
```

Docs: [website](https://shauryaR790.github.io/VPP/) · [hello-world guide](docs/getting-started/hello-world.md)

## What works

- `vpp run` interpreter + `vpp build` native codegen
- LSP, debugger (F5), tests, fmt, packages (`vpp.toml`)
- VS Code extension on Marketplace
- Windows installer with bundled LLVM; Unix from source

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
| Releases | https://github.com/shauryaR790/VPP/releases |
| Extension | https://marketplace.visualstudio.com/items?itemName=vpp-lang.vplusplus |
| Issues | https://github.com/shauryaR790/VPP/issues |
| CMake | [docs/guides/cmake.md](docs/guides/cmake.md) |
