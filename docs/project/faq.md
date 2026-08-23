# FAQ

## What is v++?

A statically typed language with readable syntax, an interpreter for development, and native compilation to `.exe` via LLVM. Open source (MIT). **v1.0.4** (compiler + extension).

## How is v++ different from Python?

| | Python | v++ |
|---|--------|-----|
| Typing | Dynamic (optional hints) | Static + inference |
| Run | Interpreter default | Interpreter + native `.exe` |
| Errors | Mostly at runtime | Types and exhaustiveness at compile time |
| Ecosystem | Huge (PyPI) | Small stdlib + growing registry |
| Best for | Scripts, ML, web backends | Learning, CLI tools, typed native programs |

Python is better when you need libraries and speed of prototyping across domains. v++ is better when you want types and a native binary without switching languages later.

## How is v++ different from Rust?

Rust enforces ownership and borrowing at compile time for maximum safety and performance. v++ uses ARC for heap values in native mode and skips the borrow checker  -  easier to learn, less control over allocation patterns.

Choose Rust for systems programming at scale. Choose v++ to learn typed languages or ship small native tools with Python-like syntax.

## How is v++ different from Go or TypeScript?

Go is statically typed with a large ecosystem and goroutines. TypeScript adds types to JavaScript but still runs on Node/V8. v++ is a standalone language with its own interpreter and LLVM backend  -  not a host-VM language.

## How do I learn v++?

1. Read [Introduction](../getting-started/introduction.md).
2. [Install](../getting-started/install.md) and write the [first program](../getting-started/hello-world.md).
3. Complete the [20 projects](../../projects/README.md).
4. Reference [language docs](../language/README.md) and [guides](../guides/README.md).

If you know Python or JavaScript, expect a few days for syntax; a few weeks for types, structs, and native builds.

## What are the main difficulties?

- **Strict types**  -  function signatures and match exhaustiveness are enforced.
- **Native builds**  -  require LLVM 22 + clang; **Windows** has full installer support today.
- **Young ecosystem**  -  fewer third-party packages than Python or Rust.
- **Unix bundles**  -  Linux/macOS native tarballs are still maturing on CI.

## Why would I choose v++?

- Readable syntax with real static typing.
- Same source for interpret (`vpp run`), debug (F5), and native ship (`vpp build`).
- Integrated toolchain: fmt, test, debug, watch, packages, LSP, VS Code extension.
- Full compiler source available to read and contribute to.

## Is v++ ready for production?

**v1.0**  -  SPEC frozen, Parity Promise, debugger, Test Explorer. Best on **Windows** for native builds. Suitable for learning, personal tools, and small native programs. Not a drop-in replacement for Python or Rust at huge scale. See [roadmap](roadmap.md).

## Where do I download?

[GitHub Releases](https://github.com/shauryaR790/VPP/releases/latest)  -  **`vpp-1.0.4-setup.exe`** for Windows (portable zip on the same page).

## Which VS Code extension is official?

**v++ Language**  -  publisher `vpp-lang` (`vpp-lang.vplusplus`). Extension **1.2.1** (pairs with compiler **v1.0.4**).

## Can I contribute?

**Yes.** The project is open source (MIT). Open an [issue](https://github.com/shauryaR790/VPP/issues) or PR  -  see [CONTRIBUTING.md](../../CONTRIBUTING.md). Docs, tests, stdlib, examples, and extension polish are great starting points. Language-design changes should be discussed in an issue first.

## Does v++ collect data?

No telemetry. Static docs site, no analytics. See [PRIVACY.md](../PRIVACY.md).

## How do I report bugs?

[GitHub Issues](https://github.com/shauryaR790/VPP/issues)
