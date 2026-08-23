# Changelog

Compiler and VS Code extension versions track together; extension may patch ahead on Marketplace when a version is immutable.

## 1.2.1  -  2026-08-22

**Current Marketplace release.** Corrected changelog and docs. Pairs with compiler **v1.0.4** on GitHub Releases.

> Note: 1.2.0 remains in version history but 1.2.1 is the recommended install (same features + fixed release notes).

## 1.2.0  -  2026-08-21

Superseded by 1.2.1 for changelog accuracy. Same extension features.

## 1.0.5  -  2026-08-22

Changelog-only upload; not shown as latest because Marketplace semver prefers 1.2.x over 1.0.x.

## 1.0.4  -  2026-08-22

**Current release.**

### Compiler
- **CMake integration**  -  `cmake/FindVpp.cmake` and `cmake/Vpp.cmake` bundled in Windows installer and portable zip
- Frozen SPEC v1.0 + Parity Promise

### Extension
- Version aligned with compiler (was briefly tagged 1.2.0 internally; Marketplace release is **1.0.4**)
- F5 debug, Test Explorer, watch, REPL, bench, LSP, registry search
- Format-on-save fix (requires `vpp` ≥ 1.0.1)

## 1.0.3  -  2026-08-21

### Compiler
- GitHub Releases publish immediately after Windows build (no longer blocked on Linux/macOS)

### Extension
- Pairs with `vpp-1.0.3-setup.exe` on GitHub Releases

## 1.0.2  -  2026-08-21

### Compiler
- Release workflow reliably publishes Windows installer + zip
- Unix LLVM link flags from `llvm-config`

## 1.0.1  -  2026-08-21

### Fixed
- **`vpp fmt`** no longer corrupts `let mut x` or range syntax like `1..6`
- Rebuild or install `vpp` ≥ 1.0.1 before relying on format-on-save in the extension

## 1.0.0  -  2026-08-21

**Stable release**  -  Parity Promise frozen.

### Compiler
- Frozen SPEC v1.0 + compatibility CI on all examples
- Full CLI: run, repl, watch, debug, bench, build, test, search, doctor, packages

### Extension
- Debug (F5), Test Explorer, registry search, LSP

## 0.9.0  -  2026-08-20

- Test Explorer UI (`vpp test --list` JSON)
- `vpp search` for package registry
- SPEC v1.0 release candidate

## 0.8.0  -  2026-08-19

- **`vpp debug`** CLI  -  step, break, locals
- VS Code debug launch (F5) + Ctrl+F5 run without debugging

## 0.7.0  -  2026-08-18

- **`vpp watch`**  -  re-runs on save
- **`vpp bench`**  -  interpreter timing
- **`vpp doctor`**  -  environment checks
- Watch File and Benchmark File commands in VS Code

## 0.6.2  -  2026-08-17

Initial Marketplace release: LSP, format-on-save, snippets, syntax highlighting.
