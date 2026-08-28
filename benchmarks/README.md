# v++ benchmark suite

Reproducible performance measurements comparing **v++** (interpreter and native LLVM), **Python**, **C++**, and **Rust** on equivalent workloads.

This suite does **not** modify the v++ compiler. Results are raw measurements  -  no marketing claims.

## What is measured

| Metric | How |
|--------|-----|
| **Execution time** | .NET `Stopwatch` around process wall time (median + min/max over N runs) |
| **Compile time** | Stopwatch around build commands (compiled languages + `vpp build`) |
| **Binary size** | File size of release executable |
| **Memory (peak working set)** | Windows `PeakWorkingSet64` after run (best-effort; not RSS on Linux) |
| **v++ interpreter** | `vpp run` (full parse + typecheck + interpret each run) |
| **v++ native** | `vpp build -o …` then run `.exe` |

## Benchmarks

| # | Name | Algorithm | Notes |
|---|------|-----------|-------|
| 1 | fibonacci | Naive recursive `fib(n)` | Same algorithm; intentionally exponential |
| 2 | primes | Trial division count to `limit` | Same algorithm in all languages (not sieve  -  see fairness) |
| 3 | sorting | Selection-sort **comparison kernel** on shared data | v++ has no array index assignment; all langs run identical comparison loops |
| 4 | array_iteration | Sum `0..n-1` | |
| 5 | string_processing | Repeated string concatenation | |
| 6 | matrix_mult | Dense matrix multiply, checksum only | No 2D storage required |
| 7 | map_lookup | Parallel key/value arrays + linear search | v++ has no hash map; **all** languages use linear lookup |
| 8 | file_processing | Write then read `file_lines` lines | Uses temp file under `results/tmp/` |
| 9 | recursive | Binary recursive tree walk, depth `d` | |
| 10 | arithmetic | Tight `int` add/xor/mul loop | |

### Fairness limitations (read before comparing)

1. **Sorting**  -  v++ cannot assign `arr[i] = …`. Every language runs the same **comparison-only** selection-sort kernel (count comparisons + checksum). No in-place reordering anywhere.
2. **Map**  -  v++ has no hash map. All languages use **linear search** on parallel `keys[]` / `values[]` arrays, not `dict` / `HashMap` / `unordered_map`.
3. **Primes**  -  Trial division in all languages (not Sieve of Eratosthenes) so v++ needs no mutable flag array.
4. **v++ interpreter**  -  Each `vpp run` re-parses and re-typechecks; not comparable to Python bytecode cache or native code alone.
5. **Memory**  -  Windows peak working set only in the default script; Linux/macOS may show blank memory columns.
6. **String**  -  v++ uses ARC heap strings; repeated `+` allocates each time (same as naive Python, not same as C++ `std::string` reuse).

## Prerequisites

- **v++**  -  `vpp` on PATH, built with codegen:  
  `cargo build --release --features codegen,lsp`
- **Python**  -  3.10+ on PATH
- **C++**  -  `clang++` or `g++` with `-O3 -std=c++17`
- **Rust**  -  stable `rustc` / `cargo`
- **PowerShell**  -  5.1+ (Windows) or PowerShell 7

Optional: LLVM/clang for `vpp build` (bundled in Windows installer under `%LOCALAPPDATA%\Programs\vpp\llvm\bin`).

## Run

From repository root:

```powershell
powershell -ExecutionPolicy Bypass -File benchmarks\scripts\benchmark.ps1
```

Options:

```powershell
.\benchmarks\scripts\benchmark.ps1 -Runs 5 -Warmup 1 -SkipBuild   # reuse existing binaries
.\benchmarks\scripts\benchmark.ps1 -Runs 3 -Benchmarks fibonacci,primes   # subset
```

Outputs:

- `benchmarks/results/latest.csv`  -  raw rows
- `benchmarks/results/latest.md`  -  summary tables
- `benchmarks/results/run-<timestamp>/`  -  archived copy + `environment.json`

## Reproduce independently

1. Verify `benchmarks/config.json` parameters match constants in each source file (grep `BENCH_`).
2. Build: run script once with `-Runs 1` or build manually (see script `Build-All`).
3. Run any single benchmark:

```powershell
# example: native v++
vpp build benchmarks\vpp\fibonacci.vpp -o benchmarks\build\vpp-fibonacci.exe
Measure-Command { .\benchmarks\build\vpp-fibonacci.exe }
```

4. Compare checksum printed to stdout against other languages (should match).

## Directory layout

```
benchmarks/
  README.md
  config.json
  scripts/benchmark.ps1
  vpp/          # .vpp sources
  python/
  cpp/
  rust/         # Cargo workspace (release binaries)
  build/        # generated executables (gitignored)
  results/
```

## After changes

Run the full v++ test suite from repo root:

```powershell
cargo test --all-targets
cargo test --features codegen --all-targets
```

Benchmark code lives outside `src/` and does not change compiler behavior.
