# Version numbers

Compiler and VS Code extension share the **3.0.0** line for this release.

| Product | Version | Where |
|---------|---------|--------|
| **Compiler** (`vpp`) | **v3.0.0** | [GitHub Releases](https://github.com/shauryaR790/VPP/releases) |
| **VS Code extension** | **3.0.0** | [Marketplace](https://marketplace.visualstudio.com/items?itemName=vpp-lang.vplusplus) |

Install both for the full toolchain: shrink/tokens, run, debug, LSP, Test Explorer.

## Releasing (maintainers)

1. Bump `Cargo.toml` and `editor/vscode-vpp/package.json` together when you want aligned semver
2. Commit → `git tag vX.Y.Z` → `git push origin vX.Y.Z`
3. CI builds installer, zip, and `vplusplus-{ext}.vsix` → GitHub Release
4. Upload the VSIX to Marketplace (or `vsce publish`)

Local fallback: `.\scripts\publish-release.ps1 -Version X.Y.Z` → `manual-releases/vX.Y.Z/`
