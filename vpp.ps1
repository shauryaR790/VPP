# Run v++ (prefers downloaded runtime — no Rust required)
$ErrorActionPreference = "Stop"

$env:VPP_HOME = $PSScriptRoot

function Test-VppExe([string]$Path) {
    if (-not (Test-Path $Path)) { return $false }
    & $Path --version *> $null
    return $LASTEXITCODE -eq 0
}

function Ensure-LlvmPath([string]$Near) {
    $roots = @()
    if ($Near) { $roots += (Split-Path $Near -Parent) }
    $roots += @(
        $PSScriptRoot,
        (Join-Path $env:LOCALAPPDATA "Programs\vpp")
    )
    foreach ($root in $roots) {
        $llvmBin = Join-Path $root "llvm\bin"
        if (Test-Path $llvmBin) {
            $env:LLVM_SYS_221_PREFIX = Split-Path $llvmBin -Parent
            $env:PATH = "$llvmBin;$env:PATH"
            return
        }
    }
    foreach ($llvm in @("C:\LLVM\bin", "C:\Program Files\LLVM\bin")) {
        if (Test-Path $llvm) {
            $env:LLVM_SYS_221_PREFIX = Split-Path $llvm -Parent
            $env:PATH = "$llvm;$env:PATH"
            return
        }
    }
}

function Find-VppExe {
    $candidates = @(
        (Join-Path $PSScriptRoot ".vpp-bin\vpp.exe"),
        (Join-Path $env:LOCALAPPDATA "Programs\vpp\vpp.exe"),
        (Join-Path $PSScriptRoot "target\release\vpp.exe"),
        (Join-Path $PSScriptRoot "target\debug\vpp.exe")
    )
    $binRoot = Join-Path $PSScriptRoot ".vpp-bin"
    if (Test-Path $binRoot) {
        Get-ChildItem $binRoot -Directory -Filter "vpp-v*-windows-x64" -ErrorAction SilentlyContinue | ForEach-Object {
            $candidates += (Join-Path $_.FullName "vpp.exe")
        }
    }
    foreach ($c in $candidates) {
        if (Test-VppExe $c) { return $c }
    }
    $onPath = Get-Command vpp -ErrorAction SilentlyContinue
    if ($onPath -and (Test-VppExe $onPath.Source)) { return $onPath.Source }
    return $null
}

$bin = Find-VppExe
if (-not $bin) {
    Write-Host "v++ not found. Running setup (downloads prebuilt compiler, no Rust needed)..." -ForegroundColor Yellow
    & (Join-Path $PSScriptRoot "setup.ps1") -SkipExtension -SkipTest
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    $bin = Find-VppExe
    if (-not $bin) {
        throw "Setup finished but vpp.exe was not found. Run .\setup.ps1 manually."
    }
}

Ensure-LlvmPath $bin
& $bin @args
exit $LASTEXITCODE
