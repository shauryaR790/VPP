# Put v++ on your user PATH (uses downloaded runtime or existing install)
$ErrorActionPreference = "Stop"

Write-Host "`n=== Installing v++ on PATH ===" -ForegroundColor Cyan

$setup = Join-Path $PSScriptRoot "setup.ps1"
if (-not (Test-Path $setup)) {
    Write-Host "setup.ps1 not found." -ForegroundColor Red
    exit 1
}

& $setup -SkipExtension -SkipTest
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

function Find-BinDir {
    $direct = Join-Path $PSScriptRoot ".vpp-bin"
    if (Test-Path (Join-Path $direct "vpp.exe")) { return $direct }
    if (Test-Path $direct) {
        foreach ($d in Get-ChildItem $direct -Directory -Filter "vpp-v*-windows-x64" -ErrorAction SilentlyContinue) {
            if (Test-Path (Join-Path $d.FullName "vpp.exe")) { return $d.FullName }
        }
    }
    $installed = Join-Path $env:LOCALAPPDATA "Programs\vpp"
    if (Test-Path (Join-Path $installed "vpp.exe")) { return $installed }
    return $null
}

$binDir = Find-BinDir
if (-not $binDir) {
    Write-Host "vpp.exe not found after setup." -ForegroundColor Red
    exit 1
}

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$paths = @($binDir)
$llvmBin = Join-Path $binDir "llvm\bin"
if (Test-Path $llvmBin) { $paths += $llvmBin }

$updated = $userPath
foreach ($p in $paths) {
    if ($updated -notlike "*$p*") {
        $updated = if ($updated) { "$updated;$p" } else { $p }
    }
}

if ($updated -ne $userPath) {
    [Environment]::SetEnvironmentVariable("Path", $updated, "User")
    Write-Host "Added to user PATH:" -ForegroundColor Green
    foreach ($p in $paths) { Write-Host "  $p" -ForegroundColor Gray }
} else {
    Write-Host "PATH already contains v++." -ForegroundColor Green
}

Write-Host "`nClose and reopen your terminal, then:" -ForegroundColor Green
Write-Host "  vpp run examples\hello.vpp" -ForegroundColor White
Write-Host "  vpp doctor" -ForegroundColor White
Write-Host ""
