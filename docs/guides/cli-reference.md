# CLI reference

```
vpp run [file.vpp]     Run via interpreter (calls main if present)
vpp build [file] -o X  Compile native executable
vpp check [file]       Type-check only
vpp compile [file]     Emit LLVM IR (.ll)
vpp fmt [file]         Format source in place
vpp shrink [file]      Minify for LLM corpora (in place)
vpp tokens [file]      Lexer-symbol token estimate
vpp test               Run test blocks in project
vpp init [name]        Create new project scaffold
vpp new NAME --path P  Create project at path
vpp add NAME ...       Add dependency to vpp.toml
vpp remove NAME        Remove dependency
vpp update             Resolve / refresh lockfile
vpp doctor             Toolchain diagnostics
vpp lsp                Start language server (stdio)
```

## Global flags

Run `vpp --help` for current options.

## Environment

| Variable | Purpose |
|----------|---------|
| `LLVM_SYS_221_PREFIX` | LLVM install path for `vpp build` (dev builds) |
| `VPP_HOME` | Optional install root (release bundles) |

## Examples

```powershell
vpp run examples/hello.vpp
vpp check examples/lesson02_functions.vpp
vpp build examples/hello.vpp -o hello.exe
vpp test
vpp doctor
```
