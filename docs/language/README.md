# v++ language overview

v++ combines readable syntax with static typing and optional native compilation.

## Design goals

- **Readable**  -  minimal punctuation, familiar control flow
- **Typed**  -  catch errors before run; infer locals, explicit function signatures
- **Fast iteration**  -  `vpp run` interpreter for development
- **Native when needed**  -  `vpp build` produces standalone `.exe` files

## Quick syntax map

| Topic | Doc |
|-------|-----|
| Types & `let` | [types-and-inference.md](types-and-inference.md) |
| Functions | [functions.md](functions.md) |
| `if` / loops / `match` | [control-flow.md](control-flow.md) |
| Structs & enums | [structs-and-enums.md](structs-and-enums.md) |
| Option / Result | [option-result-match.md](option-result-match.md) |
| Generics | [generics.md](generics.md) |
| Traits | [traits.md](traits.md) |
| `mut` | [mut-and-immutability.md](mut-and-immutability.md) |
| Modules | [modules.md](modules.md) |

Formal spec: [SPEC.md](../../SPEC.md).
