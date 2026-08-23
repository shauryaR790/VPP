# Package Windows release assets + collect VSIX for manual GitHub upload.
# Output: manual-releases/<version>/

$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
Set-Location $Root

$Versions = @("0.8.0", "0.9.0", "1.0.0", "1.0.1", "1.0.2", "1.0.3", "1.0.4")
$OutRoot = Join-Path $Root "manual-releases"
New-Item -ItemType Directory -Force -Path $OutRoot | Out-Null

Write-Host "Building vpp + vppls (release)..." -ForegroundColor Cyan
cargo build --release --features codegen,lsp --bin vpp --bin vppls
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

Write-Host "Staging portable bundle..." -ForegroundColor Cyan
$staging = Join-Path $Root "staging"
if (Test-Path $staging) { Remove-Item $staging -Recurse -Force }
New-Item -ItemType Directory -Force -Path "$staging/examples", "$staging/llvm/bin" | Out-Null
Copy-Item target/release/vpp.exe, target/release/vppls.exe $staging/
Copy-Item -Recurse std, registry, runtime $staging/
Copy-Item -Recurse cmake $staging/
Copy-Item examples/hello.vpp $staging/examples/
Copy-Item LICENSE $staging/

$llvmBin = "C:\LLVM\bin"
if (-not (Test-Path $llvmBin)) { $llvmBin = "C:\Program Files\LLVM\bin" }
if (Test-Path $llvmBin) {
    Copy-Item "$llvmBin\clang*.exe", "$llvmBin\lld*.exe", "$llvmBin\LLVM*.dll", "$llvmBin\lib*.dll" `
        -ErrorAction SilentlyContinue $staging/llvm/bin/
}

# Inno Setup (optional — for .exe installer)
$iscc = $null
foreach ($p in @("C:\Inno Setup 6\ISCC.exe", "C:\InnoSetup6\ISCC.exe")) {
    if (Test-Path $p) { $iscc = $p; break }
}
if (-not $iscc) {
    Write-Host "Downloading Inno Setup 6..." -ForegroundColor Yellow
    $issInstaller = Join-Path $env:TEMP "innosetup-6.7.3.exe"
    curl.exe -L -o $issInstaller "https://github.com/jrsoftware/issrc/releases/download/is-6_7_3/innosetup-6.7.3.exe"
    Start-Process -FilePath $issInstaller -ArgumentList "/VERYSILENT", "/SUPPRESSMSGBOXES", "/DIR=C:\InnoSetup6" -Wait
    $iscc = "C:\InnoSetup6\ISCC.exe"
}

New-Item -ItemType Directory -Force -Path (Join-Path $Root "output") | Out-Null

function Write-Sha256($filePath) {
    $hash = (Get-FileHash $filePath -Algorithm SHA256).Hash
    $name = Split-Path $filePath -Leaf
    "$hash  $name" | Set-Content "$filePath.sha256" -Encoding ascii
}

foreach ($ver in $Versions) {
    Write-Host "`n=== v$ver ===" -ForegroundColor Green
    $verDir = Join-Path $OutRoot "v$ver"
    New-Item -ItemType Directory -Force -Path $verDir | Out-Null

    # Portable zip
    $zipName = "vpp-v$ver-windows-x64.zip"
    $dirName = "vpp-v$ver-windows-x64"
    $work = Join-Path $env:TEMP $dirName
    if (Test-Path $work) { Remove-Item $work -Recurse -Force }
    Copy-Item -Recurse $staging $work
    Copy-Item GO.bat, RELEASE.txt, START-HERE.txt $work/
    $zipPath = Join-Path $verDir $zipName
    if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
    Compress-Archive -Path $work -DestinationPath $zipPath -Force
    Write-Sha256 $zipPath
    Remove-Item $work -Recurse -Force

    # Installer
    if (Test-Path $iscc) {
        Push-Location (Join-Path $Root "installer")
        & $iscc "/DMyAppVersion=$ver" vpp-setup.iss
        if ($LASTEXITCODE -ne 0) { throw "ISCC failed for v$ver" }
        Pop-Location
        $setup = Join-Path $Root "output/vpp-$ver-setup.exe"
        if (Test-Path $setup) {
            Copy-Item $setup $verDir/
            Write-Sha256 (Join-Path $verDir "vpp-$ver-setup.exe")
        }
    }

    # VSIX — package if missing
    $vsix = Join-Path $Root "vplusplus-$ver.vsix"
    if (-not (Test-Path $vsix)) {
        $pkgJson = Join-Path $Root "editor/vscode-vpp/package.json"
        $backup = Get-Content $pkgJson -Raw
        try {
            (Get-Content $pkgJson -Raw) -replace '"version":\s*"[^"]+"', "`"version`": `"$ver`"" | Set-Content $pkgJson -NoNewline
            Push-Location (Join-Path $Root "editor/vscode-vpp")
            if (-not (Get-Command vsce -ErrorAction SilentlyContinue)) {
                npm install -g @vscode/vsce 2>&1 | Out-Null
            }
            vsce package -o $vsix 2>&1 | Out-Null
            Pop-Location
        } finally {
            Set-Content $pkgJson $backup -NoNewline
        }
    }
    if (Test-Path $vsix) {
        Copy-Item $vsix $verDir/
    }

    Get-ChildItem $verDir | ForEach-Object { Write-Host "  $($_.Name)  ($([math]::Round($_.Length/1MB, 1)) MB)" }
}

Write-Host "`nDone. Upload each folder to GitHub Releases:" -ForegroundColor Cyan
Write-Host "  $OutRoot" -ForegroundColor White
Write-Host @"

For each version (v0.8.0 … v1.0.4):
  1. https://github.com/shauryaR790/VPP/releases/new
  2. Choose tag: vX.Y.Z
  3. Title: vX.Y.Z
  4. Drag ALL files from manual-releases/vX.Y.Z/
  5. Publish release

VSIX → also upload to Marketplace (not GitHub Releases).
"@
