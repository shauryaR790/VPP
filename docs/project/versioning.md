# Version numbers

Compiler and VS Code extension **version numbers differ** (extension uses the 1.2.x Marketplace line).

| Product | Version | Where |
|---------|---------|--------|
| **Compiler** (`vpp`) | **v1.0.5** | [GitHub Releases](https://github.com/shauryaR790/VPP/releases) |
| **VS Code extension** | **1.2.3** | [Marketplace](https://marketplace.visualstudio.com/items?itemName=vpp-lang.vplusplus) |

Install both for the full toolchain: `vpp` CLI + debug, LSP, Test Explorer.

## Releasing (maintainers)

1. Bump `Cargo.toml` for compiler; bump `editor/vscode-vpp/package.json` for extension (often different semver)
2. Commit → `git tag vX.Y.Z` → `git push origin vX.Y.Z`
3. CI builds installer, zip, and `vplusplus-{ext}.vsix` → GitHub Release
4. Upload the VSIX to Marketplace

Local fallback: `.\scripts\publish-release.ps1 -Version X.Y.Z` → `manual-releases/vX.Y.Z/`
