# LLM roadmap (v3.0+)

**North star:** v++ as a language optimized for **speed**, **small corpora**, and **low token use** in LLM training and tool loops — not general-purpose automation alone.

## Shipped in v3.0.0

| Tool | Purpose |
|------|---------|
| `vpp shrink` | Minify `.vpp` in place (drop layout noise; same semantics) |
| `vpp tokens` | Lexer-symbol count for context budgeting |
| `std/llm` | `est_text_tokens`, `est_bytes_tokens` |
| Docs + extension copy | LLM-first positioning on GitHub and Marketplace |

## Next (v3.x)

- **Corpus pipeline** — batch `shrink` + token report for `examples/` and stdlib
- **Compact syntax profile** — optional shorter keywords / import forms (spec-gated)
- **Token density benchmarks** — tokens-per-task on canonical v++ snippets
- **Training export** — stable JSONL of (prompt, v++ solution) pairs from tests

## Principles

1. **Measure tokens** before adding surface area to the language or stdlib.
2. **Prefer deletion** over new keywords; shrink beats features.
3. **Keep parity** — shrunk source must `vpp check` the same as pretty source.

See also [PARITY_PROMISE.md](PARITY_PROMISE.md).
