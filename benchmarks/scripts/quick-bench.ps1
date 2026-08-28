#Requires -Version 5.1
# One quick benchmark pass - Python vs C++ vs v++ native.
# Prints results to the terminal. No CSV, no 10-benchmark marathon.

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Set-Location $RepoRoot

$BenchRoot = Join-Path $RepoRoot "benchmarks"
$BuildDir = Join-Path $BenchRoot "build"
New-Item -ItemType Directory -Force -Path $BuildDir, (Join-Path $BenchRoot "results\tmp") | Out-Null

foreach ($llvm in @("C:\Program Files\LLVM", "C:\LLVM")) {
    if (Test-Path "$llvm\bin") {
        $env:LLVM_SYS_221_PREFIX = $llvm
        $env:PATH = "$llvm\bin;" + $env:PATH
        break
    }
}

$Benchmarks = @(
    @{ name = "array_sum"; label = "Array sum (5M ints)"; vpp = "array_iteration.vpp"; py = "array_iteration.py"; cpp = "array_iteration.cpp" },
    @{ name = "matrix";    label = "Matrix 128x128";    vpp = "matrix_mult.vpp";    py = "matrix_mult.py";    cpp = "matrix_mult.cpp" }
)

function Find-Vpp {
    foreach ($p in @(
        (Join-Path $RepoRoot "target\release\vpp.exe"),
        (Join-Path $RepoRoot "target\debug\vpp.exe")
    )) { if (Test-Path $p) { return $p } }
    throw "vpp not found - run: cargo build --release --features codegen,lsp"
}

function Find-Cpp {
    foreach ($n in @("clang++", "g++")) {
        $c = Get-Command $n -ErrorAction SilentlyContinue
        if ($c) { return @{ exe = $c.Source; flags = @("-O3", "-std=c++17") } }
    }
    throw "clang++ or g++ not found"
}

function Time-Exe($exe, [string[]]$cmdArgs = @()) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    if ($cmdArgs.Count -gt 0) {
        & $exe @cmdArgs 2>&1 | Out-Null
    } else {
        & $exe 2>&1 | Out-Null
    }
    $code = $LASTEXITCODE
    $sw.Stop()
    if ($code -ne 0) { throw "$exe failed (exit $code)" }
    return [math]::Round($sw.Elapsed.TotalMilliseconds, 1)
}

$vpp = Find-Vpp
$cpp = Find-Cpp
$py = (Get-Command python -ErrorAction Stop).Source

Write-Host ""
Write-Host "v++ quick benchmark (1 run each)" -ForegroundColor Cyan
Write-Host "vpp: $($vpp -replace [regex]::Escape($RepoRoot), '.')"
Write-Host "python: $py"
Write-Host "cpp: $($cpp.exe) $($cpp.flags -join ' ')"
Write-Host ""

Write-Host "Building..." -ForegroundColor DarkGray
foreach ($b in $Benchmarks) {
    $cppOut = Join-Path $BuildDir "cpp-$($b.name).exe"
    & $cpp.exe @($cpp.flags + @("-o", $cppOut, (Join-Path $BenchRoot "cpp\$($b.cpp)")))
    if ($LASTEXITCODE -ne 0) { throw "C++ build failed: $($b.name)" }

    $vppOut = Join-Path $BuildDir "vpp-$($b.name).exe"
    & $vpp build (Join-Path $BenchRoot "vpp\$($b.vpp)") -o $vppOut
    if ($LASTEXITCODE -ne 0) { throw "vpp build failed: $($b.name)" }
}

$rows = @()
foreach ($b in $Benchmarks) {
    Write-Host "  $($b.label)..." -ForegroundColor Green

    $pyMs = Time-Exe $py @((Join-Path $BenchRoot "python\$($b.py)"))
    Write-Host "    python       ${pyMs} ms"

    $cppMs = Time-Exe (Join-Path $BuildDir "cpp-$($b.name).exe")
    Write-Host "    c++          ${cppMs} ms"

    $vnMs = Time-Exe (Join-Path $BuildDir "vpp-$($b.name).exe")
    Write-Host "    v++ native   ${vnMs} ms"
    Write-Host ""

    $rows += [pscustomobject]@{
        Benchmark    = $b.label
        Python_ms    = $pyMs
        Cpp_ms       = $cppMs
        VppNative_ms = $vnMs
    }
}

Write-Host ("=" * 56) -ForegroundColor Cyan
Write-Host ("{0,-28} {1,10} {2,10} {3,10}" -f "Benchmark", "Python", "C++", "v++ native")
Write-Host ("-" * 56)
foreach ($r in $rows) {
    Write-Host ("{0,-28} {1,10} {2,10} {3,10}" -f $r.Benchmark, $r.Python_ms, $r.Cpp_ms, $r.VppNative_ms)
}
Write-Host ("=" * 56) -ForegroundColor Cyan
Write-Host ""
Write-Host "  v++ native = LLVM release. C++ = -O3." -ForegroundColor DarkGray
Write-Host ""
