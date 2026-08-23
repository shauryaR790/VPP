# First program

Create a file named `hello.vpp`:

```vpp
fn main() -> int {
    print("v++ program started")
    return 0
}
```

Every executable entry point is `fn main() -> int`. The return value is the process exit code (`0` = success).

## Run (interpreter)

```powershell
vpp run hello.vpp
```

Uses the built-in interpreter  -  no compile step. Best for learning and quick iteration.

## Type-check only

```powershell
vpp check hello.vpp
```

Reports type errors without running. Use this in CI or before committing.

## Compile to native

```powershell
vpp build hello.vpp -o hello.exe
.\hello.exe
```

Requires LLVM and clang on your PATH. Produces a standalone `.exe` with no v++ runtime dependency.

## Commands summary

| Command | Purpose |
|---------|---------|
| `vpp run` | Execute via interpreter |
| `vpp check` | Static analysis only |
| `vpp build` | Native executable via LLVM |

Next: [Your first project](first-project.md).
