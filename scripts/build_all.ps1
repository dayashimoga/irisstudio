# Iris Studio Build Script (PowerShell)
$ErrorActionPreference = "Stop"

$FlutterCmd = if (Get-Command flutter -ErrorAction SilentlyContinue) { "flutter" } elseif (Test-Path "C:\flutter\bin\flutter.bat") { "C:\flutter\bin\flutter.bat" } else { "flutter" }

Write-Host "==> Building Iris Studio Rust Core Engine (Release)..." -ForegroundColor Cyan
cargo build --release --workspace

Write-Host "==> Deploying iris_ffi.dll to Flutter runner..." -ForegroundColor Cyan
Copy-Item target\release\iris_ffi.dll apps\flutter\ -Force
if (Test-Path apps\flutter\windows\runner) {
    Copy-Item target\release\iris_ffi.dll apps\flutter\windows\runner\ -Force
}

Write-Host "==> Resolving Flutter dependencies..." -ForegroundColor Cyan
Push-Location apps/flutter
try {
    & $FlutterCmd pub get
} finally {
    Pop-Location
}

Write-Host "==> Build complete successfully!" -ForegroundColor Green
