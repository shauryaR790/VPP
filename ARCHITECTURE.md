# v++ Compiler Architecture (v0.2)

## Overview

v++ is a statically typed language with a single front-end and two backends:

- **Interpreter**  -  tree-walking evaluation for development and teaching
- **Native compiler**  -  lowers to v++ IR, then LLVM, links the C runtime

Both backends consume the same **typed AST** produced by the type checker.

```
.vpp source
    │
    ▼
 Lexer (src/lexer)
    │
    ▼
 Parser → AST (src/parser, src/ast)
    │
    ▼
 Module loader (src/modules)  -  flat merge, path imports (v0.2)
    │
    ▼
 Type checker (src/types/check.rs) → TypedProgram
    │
    ├──────────────────────┐
    ▼                      ▼
 Interpreter           IR lower (src/ir/lower.rs)
 (src/interp)                │
    │                        ▼
    │                   v++ IR (src/ir)
    │                        │
    │                        ▼
    │                   LLVM emit (src/codegen/emit.rs)
    │                        │
    │                        ▼
    │                   C runtime (runtime/vpp_runtime.c)
    │                        │
    └──────── same semantics ─┴──► native executable
```

## Components

### Lexer / Parser

Hand-written lexer with significant newlines. Recursive-descent parser with Pratt precedence for expressions.

### Type checker

Two-pass: register types and functions, then check bodies. Uses `src/builtins` for builtin signatures.

### v++ IR

Thin intermediate representation between typed AST and LLVM. Makes memory operations, control flow, and calling conventions explicit. See `src/ir/mod.rs`.

### LLVM backend

Inkwell-based. Emits LLVM IR, invokes `clang` to produce object files, links runtime. Feature-gated: `--features codegen`.

### Runtime

C ABI documented in `MEMORY_MODEL.md`. Heap strings and arrays use ARC reference counting.

### Builtins

Single registry in `src/builtins/mod.rs` consumed by type checker, interpreter, and codegen.

## Feature parity (v0.2 target)

See `CHANGELOG.md` and `tests/parity/` for the live matrix. Native codegen must match interpreter output for all supported features.

## Version notes

- **v0.2**  -  native foundation, IR, ABI, parity tests
- **v0.3**  -  module redesign, package manager (not in v0.2)
- **v0.4**  -  generics, traits, `mut`, compile-time match exhaustiveness
