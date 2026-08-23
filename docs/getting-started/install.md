# Install v++

## Windows (recommended)

1. Download **`vpp-1.0.4-setup.exe`** from [GitHub Releases](https://github.com/shauryaR790/VPP/releases/latest).
2. Run the installer. If Windows SmartScreen appears, choose **More info → Run anyway**.
3. Open a **new** terminal:

```powershell
vpp run examples\hello.vpp
vpp doctor
```

If `vpp` is not found, add the install folder to PATH:

```powershell
$dir = "$env:LOCALAPPDATA\Programs\vpp"
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$dir;$dir\llvm\bin", "User")
```

Restart the terminal.

## CMake

The installer includes `cmake/FindVpp.cmake` and `cmake/Vpp.cmake` under your install folder (same directory as `vpp.exe`). See [CMake integration](../guides/cmake.md).

## VS Code extension

1. Extensions → search **v++ Language**
2. Publisher must be **vpp-lang** (version **1.2.1**)
3. [Marketplace link](https://marketplace.visualstudio.com/items?itemName=vpp-lang.vplusplus)

See [VS Code setup](vscode-setup.md).

## Portable zip (advanced)

Download `vpp-v1.0.4-windows-x64.zip`, extract, run `GO.bat` or add the folder to PATH manually.

## Linux / macOS

Interpreter and LSP work from source today. Prebuilt native bundles for Unix are improving  -  see [GitHub Releases](https://github.com/shauryaR790/VPP/releases) when available, or [build from source](../contributing/building-from-source.md).

## Build from source

See [Building from source](../contributing/building-from-source.md).

## Requirements

| Task | Needs |
|------|--------|
| `vpp run`, `check`, `test`, `debug`, `watch` | Installer (Windows) or built `vpp` |
| `vpp build` (native `.exe`) | Bundled `clang` in installer, or LLVM 22 |
| Hack on compiler | Rust + LLVM 22 |
