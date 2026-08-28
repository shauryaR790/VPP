# v++

A small compiled language: Python-ish syntax, static types, native binaries via LLVM.

**License:** MIT · **Version:** 2.0.0 · [Releases](https://github.com/shauryaR790/VPP/releases)

## Quick start (Windows)

**Easiest:** download **`vpp-2.0.0-setup.exe`** from [GitHub Releases](https://github.com/shauryaR790/VPP/releases/latest) — no Rust needed.

**From a git clone:**

```powershell
.\setup.ps1          # downloads prebuilt v2.0.0 + installs editor extension
.\vpp.ps1 run examples\hello.vpp
```

Developers only: `.\setup.ps1 -Dev` (builds from source; needs Rust).

Docs: [website](https://shauryaR790.github.io/VPP/) · [hello-world guide](docs/getting-started/hello-world.md)

## What works

- `vpp run` interpreter + `vpp build` native codegen
- **Workflows v2.0.0:** `std/workflow` — sequence, parallel, retry, timeout ([roadmap](docs/project/AUTOMATION_ROADMAP.md))
- Typed `process.exec`, env, dirs, structured logging
- LSP, debugger (Shift+F5), tests, fmt, packages (`vpp.toml`)
- VS Code extension **2.0.0** (F5 = run in terminal)
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
