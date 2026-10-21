# Introduction to v++

v++ is a compact statically typed language: readable `.vpp` source, a fast interpreter, native compilation through LLVM, and from **v3** a clear focus on **LLM training and minimal tokens**.

**Write it. Shrink it. Run it. Build it.**

## What v++ is

v++ is its own language — not a dialect, not a transpile target. You iterate with `vpp run`, tighten files with `shrink` and `tokens`, then ship with `vpp build` when you want a standalone binary.

The repo ships a type checker, formatter, shrinker, package manager, test runner, language server, VS Code extension, and guided projects.

## Why v++

- **One source, two speeds** — interpret while you work, compile when you ship.
- **Types before run** — static checking and exhaustive `match`.
- **LLM tooling built in** — `shrink`, `tokens`, `std/llm`.
- **Full stack in one place** — CLI, LSP, debugger, Test Explorer, **26k+** extension installs.
- **Open** — MIT, entire compiler and docs in this repository.

## How to learn v++

1. [Install](install.md)
2. [First program](hello-world.md)
3. [First project](first-project.md)
4. [20 projects](../../projects/README.md)
5. [Language overview](../language/README.md) · [FAQ](../project/faq.md)

Give it a week for syntax and types; a few more for structs, enums, generics, traits, and native builds.

## Common difficulties

| Challenge | What helps |
|-----------|------------|
| **Types feel strict** | `vpp check` and read the errors; locals infer. |
| **Native build fails** | LLVM 22 + clang; `vpp doctor`. |
| **Small registry** | Stdlib covers io, fs, json, process; vendor or contribute. |
| **Upgrades** | Pin a release; read [CHANGELOG](../../CHANGELOG.md). |
| **Windows-first** | Installers and CI lead on Windows; other platforms catching up. |

## Scope

v++ is strong for typed tools, LLM corpora, learning, and native CLI programs on Windows. Mobile, WASM, and embedded are not the current focus. See [roadmap](../project/roadmap.md).

## Next steps

- [Install](install.md)
- [First program](hello-world.md)
- [Language overview](../language/README.md)
