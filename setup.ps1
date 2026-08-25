# v++ one-time setup (no Rust required for normal use)
#   .\setup.ps1           download compiler + install VS Code/Cursor extension
#   .\setup.ps1 -Dev      build from source instead (needs Rust + LLVM)

param(
    [switch]$Dev,
    [switch]$SkipExtension,
    [switch]$SkipTest
)

$ErrorActionPreference = "Stop"

function Get-VppVersion {
    $cargo = Join-Path $PSScriptRoot "Cargo.toml"
    if (-not (Test-Path $cargo)) { return "1.0.5" }
    $line = Select-String -Path $cargo -Pattern '^\s*version\s*=\s*"([^"]+)"' | Select-Object -First 1
    if ($line) { return $line.Matches.Groups[1].Value }
    return "1.0.5"
}

function Test-VppExe([string]$Path) {
    if (-not (Test-Path $Path)) { return $false }
    & $Path --version *> $null
    return $LASTEXITCODE -eq 0
}

function Find-LocalVpp {
    $candidates = @(
        (Join-Path $PSScriptRoot ".vpp-bin\vpp.exe"),
        (Join-Path $PSScriptRoot "target\release\vpp.exe"),
        (Join-Path $PSScriptRoot "target\debug\vpp.exe"),
        (Join-Path $env:LOCALAPPDATA "Programs\vpp\vpp.exe")
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

function Ensure-LlvmPath {
    if ($env:LLVM_SYS_221_PREFIX) {
        $env:PATH = "$env:LLVM_SYS_221_PREFIX\bin;$env:PATH"
    } elseif (Test-Path "C:\Program Files\LLVM\bin") {
        $env:LLVM_SYS_221_PREFIX = "C:\Program Files\LLVM"
        $env:PATH = "C:\Program Files\LLVM\bin;$env:PATH"
    }
    $installed = Join-Path $env:LOCALAPPDATA "Programs\vpp\llvm\bin"
    if (Test-Path $installed) {
        $env:LLVM_SYS_221_PREFIX = Split-Path $installed -Parent
        $env:PATH = "$installed;$env:PATH"
    }
    $portable = Join-Path $PSScriptRoot ".vpp-bin\llvm\bin"
    if (Test-Path $portable) {
        $env:LLVM_SYS_221_PREFIX = Split-Path (Split-Path $portable -Parent) -Parent
        if (-not (Test-Path (Join-Path $env:LLVM_SYS_221_PREFIX "llvm"))) {
            $env:LLVM_SYS_221_PREFIX = Split-Path $portable -Parent
        }
        $env:PATH = "$portable;$env:PATH"
    }
    Get-ChildItem (Join-Path $PSScriptRoot ".vpp-bin") -Directory -Filter "vpp-v*-windows-x64" -ErrorAction SilentlyContinue | ForEach-Object {
        $llvmBin = Join-Path $_.FullName "llvm\bin"
        if (Test-Path $llvmBin) {
            $env:LLVM_SYS_221_PREFIX = Join-Path $_.FullName "llvm"
            $env:PATH = "$llvmBin;$env:PATH"
        }
    }
}

function Get-ReleaseDownload {
    param([string]$Version)

    $repo = "shauryaR790/VPP"
    $tag = "v$Version"
    $zipName = "vpp-$tag-windows-x64.zip"
    $primary = "https://github.com/$repo/releases/download/$tag/$zipName"

    try {
        Invoke-WebRequest -Uri $primary -Method Head -UseBasicParsing | Out-Null
        return @{ Url = $primary; Label = $tag }
    } catch {
        Write-Host "  $tag zip not on Releases yet, trying latest..." -ForegroundColor DarkYellow
    }

    $latest = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest" -UseBasicParsing
    $asset = $latest.assets | Where-Object { $_.name -like "vpp-v*-windows-x64.zip" } | Select-Object -First 1
    if (-not $asset) {
        throw "No Windows zip found on GitHub Releases."
    }
    return @{ Url = $asset.browser_download_url; Label = $latest.tag_name }
}

function Install-ReleaseBundle {
    param([string]$Version)

    $destRoot = Join-Path $PSScriptRoot ".vpp-bin"
    $dl = Get-ReleaseDownload -Version $Version
    $zipPath = Join-Path $env:TEMP (Split-Path $dl.Url -Leaf)

    Write-Host "Downloading v++ $($dl.Label) (no Rust needed)..." -ForegroundColor Yellow
    Write-Host "  $($dl.Url)" -ForegroundColor DarkGray

    try {
        Invoke-WebRequest -Uri $dl.Url -OutFile $zipPath -UseBasicParsing
    } catch {
        Write-Host "Download failed: $_" -ForegroundColor Red
        Write-Host ""
        Write-Host "Install manually:" -ForegroundColor Yellow
        Write-Host "  https://github.com/shauryaR790/VPP/releases" -ForegroundColor White
        Write-Host "  Download vpp-*-windows-x64.zip or vpp-*-setup.exe" -ForegroundColor White
        exit 1
    }

    if (Test-Path $destRoot) { Remove-Item -Recurse -Force $destRoot }
    New-Item -ItemType Directory -Path $destRoot -Force | Out-Null
    Expand-Archive -Path $zipPath -DestinationPath $destRoot -Force
    Remove-Item $zipPath -Force -ErrorAction SilentlyContinue

    Get-ChildItem -LiteralPath $destRoot -Recurse | Unblock-File -ErrorAction SilentlyContinue

    $exe = Find-LocalVpp
    if (-not $exe) {
        Write-Host "Downloaded bundle but vpp.exe was not found in .vpp-bin" -ForegroundColor Red
        exit 1
    }
    Write-Host "Installed runtime -> $exe" -ForegroundColor Green
    return $exe
}

function Build-FromSource {
    $env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
    if (-not (Test-Path "$env:USERPROFILE\.cargo\bin\cargo.exe")) {
        Write-Host "Rust not found. For source builds install https://rustup.rs" -ForegroundColor Red
        Write-Host "Normal users: run .\setup.ps1 without -Dev (downloads a prebuilt compiler)." -ForegroundColor Yellow
        exit 1
    }
    Write-Host "Building v++ from source (Dev mode)..." -ForegroundColor Yellow
    Push-Location $PSScriptRoot
    try {
        Ensure-LlvmPath
        cargo build --release --features codegen,lsp
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    } finally {
        Pop-Location
    }
    return Find-LocalVpp
}

function Install-EditorExtension {
    $extSrc = Join-Path $PSScriptRoot "editor\vscode-vpp"
    if (-not (Test-Path $extSrc)) {
        Write-Host "Extension sources not found — skip editor install." -ForegroundColor DarkYellow
        return
    }

    $pkg = Get-Content (Join-Path $extSrc "package.json") -Raw | ConvertFrom-Json
    $extVersion = $pkg.version
    $extId = "$($pkg.publisher).$($pkg.name)"
    $extTargets = @(
        (Join-Path $env:USERPROFILE ".vscode\extensions\$extId-$extVersion"),
        (Join-Path $env:USERPROFILE ".cursor\extensions\$extId-$extVersion")
    )

    Write-Host "Installing v++ editor extension ($extId@$extVersion)..." -ForegroundColor Yellow

    foreach ($legacyRoot in @(
            (Join-Path $env:USERPROFILE ".vscode\extensions"),
            (Join-Path $env:USERPROFILE ".cursor\extensions")
        )) {
        if (-not (Test-Path $legacyRoot)) { continue }
        Get-ChildItem $legacyRoot -Directory -Filter "vpp-lang.vpp-*" -ErrorAction SilentlyContinue | ForEach-Object {
            Write-Host "  removing legacy $($_.Name)" -ForegroundColor DarkYellow
            Remove-Item -Recurse -Force $_.FullName
        }
    }

    foreach ($extDest in $extTargets) {
        $parent = Split-Path $extDest -Parent
        if (-not (Test-Path $parent)) {
            Write-Host "  skip (not installed): $parent" -ForegroundColor DarkGray
            continue
        }
        if (Test-Path $extDest) { Remove-Item -Recurse -Force $extDest }
        New-Item -ItemType Directory -Path $extDest -Force | Out-Null
        robocopy $extSrc $extDest /E /XD node_modules .vscode /XF *.vsix /NFL /NDL /NJH /NJS | Out-Null
        Write-Host "  installed -> $extDest" -ForegroundColor Gray
    }
}

Write-Host "`n=== v++ setup ===" -ForegroundColor Cyan
$version = Get-VppVersion
Write-Host "Target version: $version" -ForegroundColor DarkGray

Ensure-LlvmPath
$vppExe = Find-LocalVpp

if (-not $vppExe) {
    if ($Dev) {
        $vppExe = Build-FromSource
    } else {
        $vppExe = Install-ReleaseBundle -Version $version
    }
} else {
    Write-Host "Using existing compiler: $vppExe" -ForegroundColor Green
}

if (-not $SkipExtension) {
    Install-EditorExtension
}

$env:VPP_HOME = $PSScriptRoot
& $vppExe --version

Write-Host "`nDone! In VS Code or Cursor:" -ForegroundColor Green
Write-Host '  1. Reload window (Ctrl+Shift+P -> Developer: Reload Window)' -ForegroundColor White
Write-Host '  2. Open examples\hello.vpp' -ForegroundColor White
Write-Host '  3. Press F5 or Run' -ForegroundColor White
Write-Host ""
Write-Host 'Run programs:' -ForegroundColor Green
Write-Host '  .\vpp.ps1 run examples\hello.vpp' -ForegroundColor White
Write-Host '  .\vpp.ps1 run examples\arrays.vpp' -ForegroundColor White
Write-Host ""
Write-Host 'Optional: .\install.ps1 adds vpp to your user PATH' -ForegroundColor Yellow
Write-Host 'Developers: .\setup.ps1 -Dev builds from source (needs Rust)' -ForegroundColor DarkGray
Write-Host ""

if ($SkipTest) { exit 0 }

$hello = Join-Path $PSScriptRoot "examples\hello.vpp"
if (Test-Path $hello) {
    Write-Host "Smoke test: vpp run examples\hello.vpp" -ForegroundColor Cyan
    & $vppExe run $hello
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Smoke test failed (compiler is installed — check examples\hello.vpp)." -ForegroundColor Yellow
    }
}
