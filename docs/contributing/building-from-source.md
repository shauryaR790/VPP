# Building from source

## Requirements

- Rust (stable)  -  [rustup.rs](https://rustup.rs)
- LLVM 22 + clang (for `codegen` feature)
- Git (for git dependencies in tests)

## Build

```powershell
git clone https://github.com/shauryaR790/V-.git
cd V-
cargo build --release --features codegen,lsp
```

Windows:

```powershell
$env:LLVM_SYS_221_PREFIX = "C:\Program Files\LLVM"
```

Binaries: `target/release/vpp.exe`, `target/release/vppls.exe`.

## Features

| Feature | Enables |
|---------|---------|
| *(default)* | Interpreter, checker, fmt, CLI |
| `codegen` | Native `vpp build`, LLVM |
| `lsp` | `vppls` language server |

## Install locally

```powershell
cargo install --path . --features codegen,lsp
```
