# V++ North Star — Automation Roadmap v1.0.5 → v2.0.0

> **Product thesis:** V++ is a language for building **programs that talk to other programs**.
>
> Not another Python, Rust, Go, Nim, or Mojo. The goal is automation and orchestration that is **typed, composable, and native** — without Bash/YAML/Python glue for the workflow itself.

Current V++ (compiler 1.0.x, frozen SPEC v1.0) remains the foundation. Extend it; do not rewrite it.

---

## Design rules (all releases)

1. Every feature must answer: *Does this make V++ better at programs that talk to other programs?*
2. No features for size. No copying Terraform/GitHub Actions YAML syntax.
3. Small powerful abstractions over dozens of keywords.
4. Keep V++ readable and Python-like.
5. Native performance matters; benchmark major subsystems.
6. **Interpreter/native parity** on every new API.
7. **Additive changes** — existing programs keep working.
8. Syntax changes only when stdlib design fails in real programs (v1.0.9+ gate).

---

## Release architecture

| Version | Name | Ship criteria |
|---------|------|---------------|
| **v1.0.5** | Automation foundation | Typed process I/O, env, dirs, structured errors, timeouts, logging primitives |
| **v1.0.6** | Workflows | Parallel/sequential composition, retry, cancel, failure propagation (stdlib first) |
| **v1.0.7** | Program-to-program integrations | Git, HTTP, Docker, archives — consistent typed stdlib, not compiler keywords |
| **v1.0.8** | Deployable automation | Config, secrets, profiles, structured logs, CI-friendly exit/JSON output |
| **v1.0.9** | Intelligent workflow execution | Dependency graphs, caching, incremental runs, safe auto-parallelization |
| **v1.1.0** | Performance + ecosystem | Runtime efficiency, portable integrations, tooling for third-party modules |
| **v1.1.1** | Stabilization | Benchmarks, diagnostics, docs, reliability — no random features |
| **v2.0.0** | The automation language | BUILD→TEST→PACKAGE→DEPLOY→VERIFY as one native program; compiler understands automation |

**The differentiator is v1.0.9 → v2.0:** not “we have Docker APIs,” but **the compiler/runtime understands dependencies, failures, caching, and concurrency.**

---

## Current baseline (audit summary)

| Exists | Gap |
|--------|-----|
| `Result`, `Option`, `match`, structs, modules | I/O does not use `Result` everywhere |
| `std.process.run(cmd) -> int` (shell string) | Legacy; prefer `process.exec` |
| `std.fs` read/write/exists | Directories via `std.dir` (v1.0.5+) |
| JSON string validate/escape | No typed JSON values |
| Interpreter catchable errors | Native runtime `exit(1)` on some I/O failure |
| `vpp test`, `watch`, packages | No workflow model in language |

**Implementation pattern:** primitives in C runtime → thin builtins → typed stdlib → parity tests.

---

## v1.0.5 — Automation foundation

### Goal

Replace stringly glue with typed operations and structured errors.

### Minimum scope

| Component | Delivery |
|-----------|----------|
| `ProcessOutput` struct | exit_code, stdout, stderr |
| `process.exec(program, args, opts)` | `Result<ProcessOutput, string>` — argv array, no shell |
| Environment | `env.get`, `env.set` |
| Directories | `dir.list`, `dir.exists`, `dir.create` |
| Errors | Structured `Result` from exec; string err payload |
| Logging | `logging.info/warn/log_error` → structured lines (stderr) |
| Timeouts | `ProcessOptions.timeout_ms` (interpreter; native follow-up) |
| HTTP | Basic GET/POST via runtime (v1.0.6 if lib integration slips) |

### Do not ship in v1.0.5

- Workflow syntax (`parallel {}`, `retry N {}`)
- Git/Docker wrappers (v1.0.7)
- YAML config loader (v1.0.8)

### Files touched

- `runtime/vpp_automation.c` — process argv, env, dir, logging
- `src/builtins/mod.rs`, `src/types/check.rs`, `src/interp/mod.rs`, `src/codegen/emit.rs`
- `std/process.vpp`, `std/env.vpp`, `std/dir.vpp`, `std/logging.vpp`
- `tests/automation.rs`, `examples/automation_smoke.vpp`

### Success test

```vpp
import std.process

fn main() -> int {
    let r = process.exec("git", ["--version"], process.default_options())
    match r {
        Ok(out) => {
            print(out.stdout)
            return out.exit_code
        }
        Err(e) => {
            print(e)
            return 1
        }
    }
}
```

Runs identically under `vpp run` and `vpp build`.

---

## v1.0.6 — Workflows

### Goal

Composable automation as ordinary V++ code — not YAML.

### Constraint

No closures/function values today. **Phase 1:** stdlib registry / named tasks. **Phase 2:** minimal syntax only if Phase 1 fails in real scripts.

### API targets

- `workflow.sequence(tasks)`
- `workflow.parallel(tasks)` — OS threads in runtime
- `workflow.retry(n, task)`
- `workflow.with_timeout(ms, task)`
- Cancellation token (shared flag)
- Structured failure with task name + cause

### Runtime

Fixed thread pool; no async/await in v1.0.6.

---

## v1.0.7 — Program-to-program integrations

### Goal

Typed wrappers over external systems with one convention:

`Result<Output, AutomationError>` + options (timeout, cwd, log context)

### Modules (stdlib, not compiler)

| Module | Implementation |
|--------|----------------|
| `std/git.vpp` | `process.exec("git", args)` |
| `std/docker.vpp` | `process.exec("docker", args)` |
| `std/http.vpp` | runtime HTTP client |
| `std/archive.vpp` | tar/zip via runtime or CLI |
| Extend `std/json.vpp` | field access helpers |

No vendor APIs in `src/builtins/` except shared HTTP/process primitives.

---

## v1.0.8 — Deployable automation

- `std/config.vpp` — load typed config (TOML-like or JSON strings v1)
- `std/secrets.vpp` — env + file paths, never log values
- CLI: `vpp run --profile ci --json script.vpp`
- Exit codes: 0 ok, 1 task fail, 2 config error
- Artifact checksum/copy helpers

Same `.vpp` source locally and in CI.

---

## v1.0.9 — Intelligent workflow execution

Review v1.0.5–v1.0.8 real programs. Ship only proven features:

- Dependency graph (stdlib + optional IR pass)
- Content-hash cache in `.vpp/cache/`
- Skip unchanged tasks
- Auto-parallelize independent tasks
- Failures identify operation + structured cause

**Language changes gated here:** function refs, `parallel { }` blocks, `defer` — only if stdlib proved insufficient.

---

## v1.1.0 — Performance + ecosystem

- Benchmark suite vs Python/Go on automation workloads
- Streaming I/O; bounded buffers
- Integration template for registry packages
- `vpp bench --compare` integration

---

## v1.1.1 — Stabilization

- No new features
- Parity gaps closed
- Diagnostics for automation errors
- Docs + website automation guide
- Windows/Linux/macOS CI green

---

## v2.0.0 — Definition of done

One V++ program that:

1. Reads configuration  
2. Clones Git  
3. Builds  
4. Tests  
5. Builds container  
6. HTTP deploy  
7. Retries transient failures  
8. Parallel independent tasks  
9. Structured logs  
10. Useful errors  
11. Compiles to **one native executable**  
12. **No Bash/YAML orchestration layer**

SPEC v2.0 may add proven language features from v1.0.9 experiments.

---

## Before every PR

- [ ] Existing examples + `compat-v1` CI pass  
- [ ] New APIs have parity tests  
- [ ] Old builtins unchanged  
- [ ] Benchmark if hot path  
- [ ] Docs/guides updated (not necessarily SPEC until v2.0)

---

## v1.0.5 implementation checklist

- [x] This roadmap document  
- [x] Process exec with argv + stdout/stderr + timeout (interpreter)  
- [x] Environment builtins + `std/env.vpp`  
- [x] Directory builtins + `std/dir.vpp`  
- [x] `std/logging.vpp`  
- [x] `tests/automation.rs` + `examples/automation_smoke.vpp`  
- [ ] Native process timeout in C runtime  
- [ ] HTTP (track v1.0.6 if needed)

See [roadmap.md](roadmap.md) for shipped compiler milestones.
