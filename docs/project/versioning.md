# Version numbers

Compiler and VS Code extension **use the same version** (currently **v1.0.5**).

| Product | Version | Where |
|---------|---------|--------|
| **Compiler** (`vpp`) | **v1.0.5** | [GitHub Releases](https://github.com/shauryaR790/VPP/releases) |
| **VS Code extension** | **1.0.5** | [Marketplace](https://marketplace.visualstudio.com/items?itemName=vpp-lang.vplusplus) |

Install both for the full toolchain: `vpp` CLI + debug, LSP, Test Explorer.

## Releasing (maintainers)

1. Bump `Cargo.toml` **and** `editor/vscode-vpp/package.json` to the same version
2. Commit → `git tag vX.Y.Z` → `git push origin vX.Y.Z`
3. CI builds installer, zip, and `vplusplus-X.Y.Z.vsix` → GitHub Release
4. Upload the same VSIX to Marketplace

Local fallback: `.\scripts\publish-release.ps1 -Version X.Y.Z` → `manual-releases/vX.Y.Z/`
