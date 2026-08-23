# Running tests

```powershell
# Interpreter + parser + typecheck (no LLVM)
cargo test --all-targets

# Native codegen (local LLVM required)
cargo test --features codegen -- --test-threads=1

# Parity: interpreter vs native stdout
cargo test --features codegen parity

# Stress script (Windows)
.\stress.ps1
```

CI runs on every push to `main`  -  see [GitHub Actions](https://github.com/shauryaR790/V-/actions).
