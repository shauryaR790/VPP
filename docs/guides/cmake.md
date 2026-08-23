# CMake integration

Build v++ programs with CMake alongside C/C++ tooling.

## Requirements

- CMake 3.16+
- `vpp` on `PATH` with native codegen (`cargo build --release --features codegen,lsp`)
- LLVM on `PATH` (Windows installer bundles clang under `%LOCALAPPDATA%\Programs\vpp\llvm\bin`)

## Quick start

From the repo:

```powershell
cmake -S examples/cmake-demo -B build/cmake-demo
cmake --build build/cmake-demo
.\build\cmake-demo\cmake_hello.exe
```

After installing v++ on Windows, modules are at `%LOCALAPPDATA%\Programs\vpp\cmake`:

```cmake
list(APPEND CMAKE_MODULE_PATH "$ENV{LOCALAPPDATA}/Programs/vpp/cmake")
find_package(Vpp REQUIRED)
```

## Usage in your project

Copy `cmake/Vpp.cmake` and `cmake/FindVpp.cmake`, or add the v++ repo to `CMAKE_MODULE_PATH`:

```cmake
cmake_minimum_required(VERSION 3.16)
project(myapp LANGUAGES NONE)

list(APPEND CMAKE_MODULE_PATH "path/to/v++/cmake")
find_package(Vpp REQUIRED)

vpp_add_executable(myapp src/main.vpp)
```

### vpp.toml projects

```cmake
vpp_add_project(myapp ${CMAKE_CURRENT_SOURCE_DIR})
```

Reads `entry = "..."` from `vpp.toml` and runs `vpp build` with that file.

## API

| Function | Description |
|----------|-------------|
| `find_package(Vpp)` | Locates `vpp`, sets `VPP_EXECUTABLE` and `VPP_HOME` |
| `vpp_add_executable(name file.vpp)` | Custom target + `vpp build -o` |
| `vpp_add_project(name [root])` | Build manifest entry from `vpp.toml` |

## Environment

- `VPP_EXECUTABLE`  -  override compiler path
- `VPP_HOME`  -  stdlib root (auto-detected from install or repo)
- `LLVM_SYS_221_PREFIX`  -  forwarded to `vpp build` when set

## Notes

- Each target runs `vpp build` as a custom command; incremental builds re-run when the `.vpp` source changes.
- Mixed C++/v++ linking is not supported yet  -  this wraps the v++ compiler only.
- Interpreter mode (`vpp run`) is not used; outputs are native LLVM binaries.
