# Formatting

```powershell
vpp fmt path/to/file.vpp
vpp shrink path/to/file.vpp   # v3+: minify for LLM datasets (same semantics)
vpp tokens path/to/file.vpp   # budget lexer symbols
```

`fmt` re-indents blocks and normalizes spacing. `shrink` removes layout noise for smaller training corpora. Neither changes semantics.

Format before committing; use `shrink` when exporting v++ for LLM training. IDE format-on-save uses `vpp fmt`.
