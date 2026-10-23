# v++ 3.0.0 — LLM-first

New direction: **optimize for speed and the fewest tokens** when models read or generate v++.

## Compiler & stdlib

- **`vpp shrink`** — minify `.vpp` in place; reports byte and token savings
- **`vpp tokens`** — estimate lexer-symbol count per file
- **`std/llm`** — byte/text token estimates for scripts
- [LLM roadmap](docs/project/LLM_ROADMAP.md)

## VS Code extension 3.0.0

- Marketplace README highlights LLM tooling
- Upload **`vplusplus-3.0.0.vsix`**

## Upgrade

```powershell
.\setup.ps1
```

Or install the VSIX from this release.
