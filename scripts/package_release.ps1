# Iris Studio Production Release Automation Script
# Builds release binaries, generates SHA-256 checksums, and stages distribution bundles.
$ErrorActionPreference = "Stop"

$Version = "1.0.0"
$RootDir = Split-Path -Parent $PSScriptRoot
$DistDir = Join-Path $RootDir "dist"
$WinDist = Join-Path $DistDir "windows"

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "          IRIS STUDIO PRODUCTION RELEASE PACKAGING          " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "Version: $Version"
Write-Host "Target: Windows x64 / Cross-Platform Assets"

# 1. Clean and prepare dist directory
if (Test-Path $DistDir) {
    Remove-Item -Recurse -Force $DistDir
}
New-Item -ItemType Directory -Path $WinDist -Force | Out-Null

# 2. Compile optimized Rust release binaries
Write-Host "==> Compiling Rust Release Engine & FFI..." -ForegroundColor Cyan
Push-Location $RootDir
try {
    cargo build --release -p iris_ffi
    cargo build --release --bin run_benchmarks
} finally {
    Pop-Location
}

# 3. Stage Windows Portable Bundle
Write-Host "==> Staging Windows Portable Distribution..." -ForegroundColor Cyan
$PortableDir = Join-Path $WinDist "IrisStudio-Portable-$Version-x64"
New-Item -ItemType Directory -Path $PortableDir -Force | Out-Null

Copy-Item -Path (Join-Path $RootDir "target\release\iris_ffi.dll") -Destination $PortableDir -Force
Copy-Item -Path (Join-Path $RootDir "target\release\run_benchmarks.exe") -Destination $PortableDir -Force
Copy-Item -Path (Join-Path $RootDir "LICENSE") -Destination $PortableDir -Force
Copy-Item -Path (Join-Path $RootDir "README.md") -Destination $PortableDir -Force

# Copy Flutter application bundle if built
$FlutterRelease = Join-Path $RootDir "apps\flutter\build\windows\x64\runner\Release"
if (Test-Path $FlutterRelease) {
    Copy-Item -Path "$FlutterRelease\*" -Destination $PortableDir -Recurse -Force
} else {
    Write-Host "==> [NOTICE] Flutter desktop release not compiled yet. Staging native libraries & assets." -ForegroundColor Yellow
}

# 4. Generate ZIP archive
$ZipOutput = Join-Path $WinDist "IrisStudio-v$Version-windows-x64.zip"
Write-Host "==> Compressing into $ZipOutput..." -ForegroundColor Cyan
Compress-Archive -Path "$PortableDir\*" -DestinationPath $ZipOutput -Force

# 5. Generate SHA-256 Checksums
Write-Host "==> Computing Cryptographic Checksums (SHA-256)..." -ForegroundColor Cyan
$ChecksumFile = Join-Path $DistDir "SHA256SUMS.txt"
$Hashes = @()
Get-ChildItem -Path $WinDist -Filter "*.zip" | ForEach-Object {
    $hash = (Get-FileHash -Path $_.FullName -Algorithm SHA256).Hash.ToLower()
    $entry = "$hash  $($_.Name)"
    $Hashes += $entry
    Write-Host "  $entry" -ForegroundColor Green
}
$Hashes | Out-File -FilePath $ChecksumFile -Encoding utf8

Write-Host "============================================================" -ForegroundColor Green
Write-Host "PRODUCTION RELEASE BUILD COMPLETED SUCCESSFULLY!" -ForegroundColor Green
Write-Host "Artifacts written to: $DistDir" -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Green
