# v++ 2.1.0

Light runtime optimizations — no language changes.

## Compiler

- `vpp run -O` / `--optimize` — faster repeat loads (module parse cache) and less workflow logging via `VPP_OPT=1`
- `std/opt` — `opt.enabled()`
- `workflow.parallel` — at most 8 concurrent child processes per batch

## VS Code extension 2.1.0

- Setting **`vpp.optimizeRun`** — adds `-O` to F5 / terminal run
- Upload **`vplusplus-2.1.0.vsix`** to the Marketplace

## Upgrade

```powershell
.\setup.ps1
```

Or install the VSIX from this release.
