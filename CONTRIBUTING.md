# Contributing to v++

**Yes  -  contributions are welcome.** v++ is MIT-licensed and open source. Issues and pull requests are accepted on GitHub.

This is a young project (small maintainer team, fast-moving `main`). We review PRs as we can; small, focused changes merge faster than large rewrites.

## Good first contributions

| Area | Examples |
|------|----------|
| **Docs** | Fix typos, clarify guides, add examples |
| **Tests** | `.vpp` test cases, `cargo test` coverage |
| **Stdlib** | `std/*` modules, registry packages |
| **Extension** | `editor/vscode-vpp/`  -  UX, snippets, docs |
| **Examples** | `examples/`, `projects/` |
| **Bug fixes** | Link a repro in the issue first |

Language-design changes (new syntax, breaking SPEC)  -  **open an issue first** so we can discuss before you invest time.

## Quick links

- [Documentation hub](docs/README.md)
- [Build from source](docs/contributing/building-from-source.md)
- [Run tests](docs/contributing/running-tests.md)
- [Report a bug](https://github.com/shauryaR790/VPP/issues/new)
- [Contributing guide](CONTRIBUTING.md) · [GitHub](https://github.com/shauryaR790/VPP)

## How to contribute

1. **Issues first**  -  open an issue for bugs or feature ideas before large PRs
2. **Fork & branch**  -  work on a feature branch off `main`
3. **Test**  -  `cargo test --all-targets`; if touching codegen, `cargo test --features codegen`
4. **Format**  -  `cargo fmt`, `vpp fmt` on any `.vpp` examples you change
5. **PR**  -  describe what changed and why; link the issue

## Code areas

| Area | Path |
|------|------|
| Lexer / parser | `src/lexer`, `src/parser` |
| Type checker | `src/types` |
| Interpreter | `src/interp` |
| Codegen | `src/codegen` |
| LSP | `src/lsp`, `src/bin/vppls.rs` |
| VS Code extension | `editor/vscode-vpp/` |
| Docs | `docs/` |

## Commit messages

Use clear summaries: `Fix …`, `Add …`, `Docs: …`  -  same style as existing history.

## Releases (maintainers)

1. Bump **the same version** in `Cargo.toml` and `editor/vscode-vpp/package.json`
2. Update `CHANGELOG.md` and `editor/vscode-vpp/CHANGELOG.md`
3. Tag and push: `git tag vX.Y.Z && git push origin main && git push origin vX.Y.Z`
4. Confirm the [Release workflow](https://github.com/shauryaR790/VPP/actions/workflows/release.yml) succeeds on [GitHub Releases](https://github.com/shauryaR790/VPP/releases)

## License

By contributing, you agree your work is licensed under the project MIT license.
