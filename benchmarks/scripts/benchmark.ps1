#Requires -Version 5.1
<#
.SYNOPSIS
  Build and run the v++ cross-language benchmark suite.
#>
param(
    [int]$Runs = 5,
    [int]$Warmup = 1,
    [switch]$SkipBuild,
    [string[]]$Benchmarks = @()
)

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Set-Location $RepoRoot

$BenchRoot = Join-Path $RepoRoot "benchmarks"
$BuildDir = Join-Path $BenchRoot "build"
$ResultsDir = Join-Path $BenchRoot "results"
$TmpDir = Join-Path $ResultsDir "tmp"
$ConfigPath = Join-Path $BenchRoot "config.json"

# LLVM required for release vpp.exe on Windows
foreach ($llvm in @("C:\Program Files\LLVM", "C:\LLVM")) {
    if (Test-Path "$llvm\bin") {
        $env:LLVM_SYS_221_PREFIX = $llvm
        $env:PATH = "$llvm\bin;" + $env:PATH
        break
    }
}

$AllBenchmarks = @(
    "fibonacci", "primes", "sorting", "array_iteration", "string_processing",
    "matrix_mult", "map_lookup", "file_processing", "recursive", "arithmetic"
)

if ($Benchmarks.Count -eq 1 -and $Benchmarks[0] -match ",") {
    $Benchmarks = $Benchmarks[0] -split "," | ForEach-Object { $_.Trim() }
}

if ($Benchmarks.Count -eq 0) {
    $Benchmarks = $AllBenchmarks
}

New-Item -ItemType Directory -Force -Path $BuildDir, $ResultsDir, $TmpDir | Out-Null

function Read-Config {
    if (Test-Path $ConfigPath) {
        return Get-Content $ConfigPath -Raw | ConvertFrom-Json
    }
    return $null
}

function Get-Stats([double[]]$Values) {
    $sorted = $Values | Sort-Object
    $count = $sorted.Count
    if ($count -eq 0) { return @{ median = 0; min = 0; max = 0; mean = 0 } }
    $mid = [math]::Floor(($count - 1) / 2)
    $median = if ($count % 2 -eq 1) { $sorted[$mid] } else { ($sorted[$mid] + $sorted[$mid + 1]) / 2.0 }
    return @{
        median = [math]::Round($median, 3)
        min    = [math]::Round($sorted[0], 3)
        max    = [math]::Round($sorted[$count - 1], 3)
        mean   = [math]::Round(($sorted | Measure-Object -Average).Average, 3)
    }
}

function Find-Vpp {
    foreach ($p in @(
        (Join-Path $RepoRoot "target\release\vpp.exe"),
        (Join-Path $RepoRoot "target\debug\vpp.exe")
    )) {
        if (Test-Path $p) { return $p }
    }
    $cmd = Get-Command vpp -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    throw "vpp not found. Build with: cargo build --release --features codegen,lsp"
}

function Find-CppCompiler {
    foreach ($name in @("clang++", "g++")) {
        $cmd = Get-Command $name -ErrorAction SilentlyContinue
        if ($cmd) { return @{ exe = $cmd.Source; flags = @("-O3", "-std=c++17") } }
    }
    throw "No C++ compiler (clang++ or g++) on PATH"
}

function Get-EnvironmentInfo {
    $vpp = Find-Vpp
    $vppVer = & $vpp --version 2>&1 | Out-String
    $pyVer = try { (python --version 2>&1) } catch { "missing" }
    $rustVer = try { (rustc --version 2>&1) } catch { "missing" }
    $cargoVer = try { (cargo --version 2>&1) } catch { "missing" }
    $cpp = try { Find-CppCompiler } catch { $null }
    $cppStr = if ($cpp) { "$($cpp.exe) $($cpp.flags -join ' ')" } else { "missing" }

    $cpu = (Get-CimInstance Win32_Processor | Select-Object -First 1).Name
    $ramGb = [math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1GB, 1)
    $os = (Get-CimInstance Win32_OperatingSystem).Caption

    return [ordered]@{
        timestamp_utc = (Get-Date).ToUniversalTime().ToString("o")
        os            = $os
        cpu           = $cpu
        ram_gb        = $ramGb
        vpp           = $vppVer.Trim()
        vpp_path      = $vpp
        python        = "$pyVer".Trim()
        rust          = "$rustVer".Trim()
        cargo         = "$cargoVer".Trim()
        cpp           = $cppStr
        runs          = $Runs
        warmup        = $Warmup
        repo_root     = $RepoRoot
    }
}

function Measure-ProcessRun {
    param(
        [scriptblock]$Launch,
        [int]$Runs,
        [int]$Warmup
    )
    for ($w = 0; $w -lt $Warmup; $w++) {
        & $Launch | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "Warmup run failed (exit $LASTEXITCODE)" }
    }
    $times = New-Object System.Collections.Generic.List[double]
    $mems = New-Object System.Collections.Generic.List[long]
    for ($i = 0; $i -lt $Runs; $i++) {
        $psi = $Launch.InvokeReturn()
        if (-not $psi) {
            $sw = [System.Diagnostics.Stopwatch]::StartNew()
            & $Launch | Out-Null
            $code = $LASTEXITCODE
            $sw.Stop()
            if ($code -ne 0) { throw "Run failed (exit $code)" }
            $times.Add($sw.Elapsed.TotalMilliseconds)
            continue
        }
        $proc = Start-Process -FilePath $psi.File -ArgumentList $psi.Args -WorkingDirectory $psi.WorkDir `
            -PassThru -Wait -WindowStyle Hidden
        if ($proc.ExitCode -ne 0) { throw "Run failed (exit $($proc.ExitCode))" }
        $times.Add($proc.TotalProcessorTime.TotalMilliseconds)
        if ($proc.PeakWorkingSet64 -gt 0) { $mems.Add([long]$proc.PeakWorkingSet64) }
    }
    $stats = Get-Stats ($times.ToArray())
    $memMb = if ($mems.Count -gt 0) { [math]::Round(($mems | Measure-Object -Maximum).Maximum / 1MB, 2) } else { $null }
    return @{ stats = $stats; memory_mb = $memMb }
}

function Measure-SimpleRun {
    param(
        [string]$Exe,
        [string[]]$Args = @(),
        [string]$WorkDir = $RepoRoot,
        [int]$Runs,
        [int]$Warmup
    )
    for ($w = 0; $w -lt $Warmup; $w++) {
        if ($Args -and $Args.Count -gt 0) {
            $p = Start-Process -FilePath $Exe -ArgumentList $Args -WorkingDirectory $WorkDir -PassThru -Wait -WindowStyle Hidden
        } else {
            $p = Start-Process -FilePath $Exe -WorkingDirectory $WorkDir -PassThru -Wait -WindowStyle Hidden
        }
        if ($p.ExitCode -ne 0) { throw "Warmup failed: $Exe ($($p.ExitCode))" }
    }
    $times = @()
    $memPeak = 0L
    for ($i = 0; $i -lt $Runs; $i++) {
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        if ($Args -and $Args.Count -gt 0) {
            $p = Start-Process -FilePath $Exe -ArgumentList $Args -WorkingDirectory $WorkDir -PassThru -Wait -WindowStyle Hidden
        } else {
            $p = Start-Process -FilePath $Exe -WorkingDirectory $WorkDir -PassThru -Wait -WindowStyle Hidden
        }
        $sw.Stop()
        if ($p.ExitCode -ne 0) { throw "Run failed: $Exe ($($p.ExitCode))" }
        $times += $sw.Elapsed.TotalMilliseconds
        if ($p.PeakWorkingSet64 -gt $memPeak) { $memPeak = $p.PeakWorkingSet64 }
    }
    $stats = Get-Stats $times
    $memMb = if ($memPeak -gt 0) { [math]::Round($memPeak / 1MB, 2) } else { $null }
    return @{ stats = $stats; memory_mb = $memMb }
}

function Build-All {
    Write-Host "Building benchmarks..." -ForegroundColor Cyan
    $vpp = Find-Vpp
    $cpp = Find-CppCompiler

    foreach ($name in $AllBenchmarks) {
        $src = Join-Path $BenchRoot "cpp\$name.cpp"
        $out = Join-Path $BuildDir "cpp-$name.exe"
        $args = $cpp.flags + @("-o", $out, $src)
        & $cpp.exe @args
        if ($LASTEXITCODE -ne 0) { throw "C++ build failed: $name" }

        $vppSrc = Join-Path $BenchRoot "vpp\$name.vpp"
        $vppOut = Join-Path $BuildDir "vpp-$name.exe"
        & $vpp build $vppSrc -o $vppOut
        if ($LASTEXITCODE -ne 0) { throw "vpp build failed: $name" }
    }

    Push-Location (Join-Path $BenchRoot "rust")
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
    Pop-Location

    foreach ($name in $AllBenchmarks) {
        $rustExe = Join-Path $BenchRoot "rust\target\release\$name.exe"
        if (-not (Test-Path $rustExe)) { throw "Missing Rust binary: $name" }
        Copy-Item $rustExe (Join-Path $BuildDir "rust-$name.exe") -Force
    }
}

function Measure-CompileTimes {
    $vpp = Find-Vpp
    $cpp = Find-CppCompiler
    $rows = @()

    foreach ($name in $Benchmarks) {
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        $vppSrc = Join-Path $BenchRoot "vpp\$name.vpp"
        $vppOut = Join-Path $BuildDir "vpp-compile-$name.exe"
        & $vpp build $vppSrc -o $vppOut
        if ($LASTEXITCODE -ne 0) { throw "vpp compile timing failed: $name" }
        $sw.Stop()
        $rows += [pscustomobject]@{
            benchmark = $name; language = "vpp_native_compile"; compile_ms = [math]::Round($sw.Elapsed.TotalMilliseconds, 3)
        }

        $sw.Restart()
        $cppSrc = Join-Path $BenchRoot "cpp\$name.cpp"
        $cppOut = Join-Path $BuildDir "cpp-compile-$name.exe"
        & $cpp.exe @($cpp.flags + @("-o", $cppOut, $cppSrc))
        if ($LASTEXITCODE -ne 0) { throw "cpp compile timing failed: $name" }
        $sw.Stop()
        $rows += [pscustomobject]@{
            benchmark = $name; language = "cpp_compile"; compile_ms = [math]::Round($sw.Elapsed.TotalMilliseconds, 3)
        }
    }

    Push-Location (Join-Path $BenchRoot "rust")
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "rust compile failed" }
    $sw.Stop()
    Pop-Location
    $rows += [pscustomobject]@{
        benchmark = "all"; language = "rust_release_build"; compile_ms = [math]::Round($sw.Elapsed.TotalMilliseconds, 3)
    }
    return $rows
}

function Get-BinarySizeMb($path) {
    if (-not (Test-Path $path)) { return $null }
    return [math]::Round((Get-Item $path).Length / 1MB, 3)
}

function Add-ResultRow {
    param($Rows, $Benchmark, $Language, $Mode, $Measure, $BinaryPath)
    $Rows.Add([pscustomobject]@{
        benchmark   = $Benchmark
        language    = $Language
        mode        = $Mode
        median_ms   = $Measure.stats.median
        min_ms      = $Measure.stats.min
        max_ms      = $Measure.stats.max
        mean_ms     = $Measure.stats.mean
        memory_mb   = $Measure.memory_mb
        binary_mb   = (Get-BinarySizeMb $BinaryPath)
    }) | Out-Null
}

# --- main ---
$envInfo = Get-EnvironmentInfo
$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$runDir = Join-Path $ResultsDir "run-$timestamp"
New-Item -ItemType Directory -Force -Path $runDir | Out-Null
$envInfo | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $runDir "environment.json") -Encoding UTF8

if (-not $SkipBuild) {
    Build-All
}

$vpp = Find-Vpp
$resultRows = New-Object System.Collections.Generic.List[object]
$compileRows = Measure-CompileTimes
$compileRows | Export-Csv (Join-Path $runDir "compile_times.csv") -NoTypeInformation

Write-Host "`nRunning benchmarks (Runs=$Runs, Warmup=$Warmup)..." -ForegroundColor Cyan

foreach ($name in $Benchmarks) {
    Write-Host "  $name" -ForegroundColor Green

    if ($name -eq "file_processing") {
        Remove-Item (Join-Path $TmpDir "bench-io.txt") -ErrorAction SilentlyContinue
    }

    # Python
    $py = (Get-Command python -ErrorAction Stop).Source
    $m = Measure-SimpleRun -Exe $py -Args @((Join-Path $BenchRoot "python\$name.py")) -Runs $Runs -Warmup $Warmup
    Add-ResultRow $resultRows $name "python" "execute" $m $null

    # C++
    $cppExe = Join-Path $BuildDir "cpp-$name.exe"
    $m = Measure-SimpleRun -Exe $cppExe -Runs $Runs -Warmup $Warmup
    Add-ResultRow $resultRows $name "cpp" "execute" $m $cppExe

    # Rust
    $rustExe = Join-Path $BuildDir "rust-$name.exe"
    $m = Measure-SimpleRun -Exe $rustExe -Runs $Runs -Warmup $Warmup
    Add-ResultRow $resultRows $name "rust" "execute" $m $rustExe

    # v++ interpreter
    $vppSrc = Join-Path $BenchRoot "vpp\$name.vpp"
    $m = Measure-SimpleRun -Exe $vpp -Args @("run", $vppSrc) -Runs $Runs -Warmup $Warmup
    Add-ResultRow $resultRows $name "vpp" "interpreter" $m $null

    # v++ native
    $vppExe = Join-Path $BuildDir "vpp-$name.exe"
    $m = Measure-SimpleRun -Exe $vppExe -Runs $Runs -Warmup $Warmup
    Add-ResultRow $resultRows $name "vpp" "native" $m $vppExe
}

$rows = $resultRows.ToArray()
$rows | Export-Csv (Join-Path $runDir "results.csv") -NoTypeInformation
$rows | Export-Csv (Join-Path $ResultsDir "latest.csv") -NoTypeInformation

# Markdown report
$md = @()
$md += "# v++ benchmark results"
$md += ""
$md += "Generated: $($envInfo.timestamp_utc)"
$md += ""
$md += "## Environment"
$md += ""
$md += "| Key | Value |"
$md += "|-----|-------|"
foreach ($k in $envInfo.Keys) {
    if ($k -eq "repo_root") { continue }
    $md += "| $k | $($envInfo[$k]) |"
}
$md += ""
$md += "## Execution time (median ms)"
$md += ""
$md += "| Benchmark | Python | v++ interp | v++ native | C++ | Rust |"
$md += "|-----------|--------|------------|------------|-----|------|"

foreach ($name in $Benchmarks) {
    $get = {
        param($l, $m)
        $r = $rows | Where-Object { $_.benchmark -eq $name -and $_.language -eq $l -and $_.mode -eq $m } | Select-Object -First 1
        if ($r) { return "$($r.median_ms)" } else { return "n/a" }
    }
    $py = & $get "python" "execute"
    $vi = & $get "vpp" "interpreter"
    $vn = & $get "vpp" "native"
    $cp = & $get "cpp" "execute"
    $rs = & $get "rust" "execute"
    $md += "| $name | $py | $vi | $vn | $cp | $rs |"
}

$md += ""
$md += "## v++ interpreter vs native (median ms)"
$md += ""
$md += "| Benchmark | Interpreter | Native | Ratio (interp/native) |"
$md += "|-----------|-------------|--------|------------------------|"
foreach ($name in $Benchmarks) {
    $vi = ($rows | Where-Object { $_.benchmark -eq $name -and $_.mode -eq "interpreter" }).median_ms
    $vn = ($rows | Where-Object { $_.benchmark -eq $name -and $_.mode -eq "native" }).median_ms
    $ratio = if ($vn -gt 0) { [math]::Round($vi / $vn, 2) } else { "n/a" }
    $md += "| $name | $vi | $vn | $ratio |"
}

$md += ""
$md += "## Binary sizes (MB)"
$md += ""
$md += "| Benchmark | v++ native | C++ | Rust |"
$md += "|-----------|------------|-----|------|"
foreach ($name in $Benchmarks) {
    $vn = ($rows | Where-Object { $_.benchmark -eq $name -and $_.mode -eq "native" }).binary_mb
    $cp = ($rows | Where-Object { $_.benchmark -eq $name -and $_.language -eq "cpp" }).binary_mb
    $rs = ($rows | Where-Object { $_.benchmark -eq $name -and $_.language -eq "rust" }).binary_mb
    $md += "| $name | $vn | $cp | $rs |"
}

$md += ""
$md += "## Memory peak (MB, Windows working set)"
$md += ""
$md += "| Benchmark | Python | v++ native | C++ | Rust |"
$md += "|-----------|--------|------------|-----|------|"
foreach ($name in $Benchmarks) {
    $py = ($rows | Where-Object { $_.benchmark -eq $name -and $_.language -eq "python" }).memory_mb
    $vn = ($rows | Where-Object { $_.benchmark -eq $name -and $_.mode -eq "native" }).memory_mb
    $cp = ($rows | Where-Object { $_.benchmark -eq $name -and $_.language -eq "cpp" }).memory_mb
    $rs = ($rows | Where-Object { $_.benchmark -eq $name -and $_.language -eq "rust" }).memory_mb
    $md += "| $name | $py | $vn | $cp | $rs |"
}

$md += ""
$md += "## Compile times"
$md += ""
$md += "| Benchmark | v++ build (ms) | C++ (ms) |"
$md += "|-----------|------------------|----------|"
foreach ($name in $Benchmarks) {
    $vb = ($compileRows | Where-Object { $_.benchmark -eq $name -and $_.language -eq "vpp_native_compile" }).compile_ms
    $cb = ($compileRows | Where-Object { $_.benchmark -eq $name -and $_.language -eq "cpp_compile" }).compile_ms
    $md += "| $name | $vb | $cb |"
}
$rustAll = ($compileRows | Where-Object { $_.language -eq "rust_release_build" }).compile_ms
$md += ""
$md += "Rust release build (all binaries): **${rustAll} ms**"
$md += ""
$md += "## Fairness notes"
$md += ""
$md += "- Sorting: comparison-only selection-sort kernel (v++ lacks array index assignment)."
$md += "- Map: linear search on parallel arrays in all languages (v++ has no hash map)."
$md += "- Primes: trial division in all languages."
$md += "- v++ interpreter times include parse + typecheck each run."
$md += ""
$md += "Raw CSV: ``results/run-$timestamp/results.csv``"

$mdText = $md -join "`n"
Set-Content (Join-Path $ResultsDir "latest.md") -Value $mdText -Encoding UTF8
Set-Content (Join-Path $runDir "results.md") -Value $mdText -Encoding UTF8
Copy-Item (Join-Path $ResultsDir "latest.csv") (Join-Path $runDir "latest.csv")

Write-Host "`nDone." -ForegroundColor Cyan
Write-Host "  $($ResultsDir)\latest.md"
Write-Host "  $($ResultsDir)\latest.csv"
Write-Host "  $runDir"
