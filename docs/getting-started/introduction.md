# Introduction to v++

v++ is a statically typed programming language with readable syntax, a fast interpreter for development, and native compilation to standalone executables via LLVM.

**Write it simply. Compile it natively. Grow into control when you need it.**

## What v++ is

v++ targets developers who want Python-style readability without giving up compile-time safety or native performance. You start with `vpp run` during learning and prototyping, then ship the same source with `vpp build` when you need a `.exe` with no runtime dependency.

The toolchain includes a type checker, formatter, package manager, test runner, VS Code language server, and twenty guided projects from beginner to advanced.

## How v++ compares

| | Python | Rust | C/C++ | v++ |
|---|--------|------|-------|-----|
| Syntax | Very readable | Steeper | Verbose | Readable, minimal |
| Typing | Dynamic (optional hints) | Static, strict | Static | Static + inference |
| First run | Interpreter | Compile | Compile | Interpreter (`vpp run`) |
| Native output | No (without extra tools) | Yes | Yes | Yes (`vpp build`) |
| Learning curve | Low | High | Medium–high | Low → medium |
| Ecosystem | Massive | Large | Huge | Growing (v1.0+) |

**vs Python:** v++ catches type errors before run, produces native binaries, and keeps similar control-flow and function syntax. You trade PyPI's scale for a smaller, focused stdlib and a compiler that fits in one repo.

**vs Rust:** v++ avoids ownership/borrowing complexity upfront. Memory is ARC-managed in native builds; you get structs, enums, `match`, generics, and traits without the full Rust learning cliff.

**vs C/C++:** v++ removes manual memory management and header boilerplate. You still get native code and predictable performance for the supported subset.

## Why use v++ now

- **One language, two modes**  -  interpret while learning, compile when shipping.
- **Errors before run**  -  static types and exhaustiveness checking on `match`.
- **Real toolchain**  -  CLI, LSP, VS Code extension, packages, tests, not a toy parser.
- **Open source**  -  MIT licensed, compiler and docs in one repository.
- **Honest scope**  -  v1.0 SPEC is frozen; Windows is the primary native platform; Unix bundles improving.

## How to learn v++

1. [Install](install.md) the Windows installer or build from source.
2. Write your [first program](hello-world.md)  -  types, `fn main`, run/check/build.
3. Scaffold a [project](first-project.md) with `vpp new` and run tests.
4. Work through the [20 projects](../../projects/README.md) in order  -  each builds on the last.
5. Read the [language overview](../language/README.md) and [FAQ](../project/faq.md).
6. Use [Documentation](../../docs.html) as reference while coding.

Expect **1–2 weeks** to feel comfortable with syntax and types if you know Python or JavaScript. Expect **another few weeks** for structs, enums, generics, traits, and native builds.

## Common difficulties

| Challenge | What helps |
|-----------|------------|
| **Types feel strict** | Start with `vpp check` and read error messages; locals are inferred. |
| **Native build fails** | Install LLVM 22 + clang; run `vpp doctor`. |
| **Small ecosystem** | Stdlib covers io, fs, json, process; contribute or vendor code. |
| **Language still evolving** | Pin a release; watch [CHANGELOG](../../CHANGELOG.md) before upgrading. |
| **Windows-first** | Primary CI and installers are Windows; other platforms are best-effort. |

## When v++ is not the right choice

- You need a mature package ecosystem (use Python, Node, or Rust).
- You need mobile, WebAssembly, or embedded targets today (not primary focus).
- You need Linux/macOS prebuilt native bundles today (Windows installer is fully supported).

## Next steps

- [Install](install.md)
- [First program](hello-world.md)
- [Language overview](../language/README.md)
- [Full documentation](../README.md)
