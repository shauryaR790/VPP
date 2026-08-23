# Native compilation

`vpp build` lowers code to LLVM IR, compiles with clang, and links the v++ C runtime.

```powershell
vpp build examples/hello.vpp -o hello.exe
.\hello.exe
```

## Requirements

- **Release installer**  -  includes portable `clang` under `llvm\bin`
- **From source**  -  LLVM 22 + `LLVM_SYS_221_PREFIX`

```powershell
$env:LLVM_SYS_221_PREFIX = "C:\Program Files\LLVM"
cargo build --release --features codegen
```

## Interpreter vs native

| | `vpp run` | `vpp build` |
|---|-----------|-------------|
| Speed to iterate | Fast | Slower |
| Output | Runs in VM | Standalone `.exe` |
| LLVM needed | No | Yes |

Parity tests ensure interpreter and native produce the same stdout for key examples.

## Memory model

Strings and arrays use ARC in native code. See [MEMORY_MODEL.md](../../MEMORY_MODEL.md).
