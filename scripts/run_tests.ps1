# Iris Studio Verification Script (PowerShell)
$ErrorActionPreference = "Stop"

$FlutterCmd = if (Get-Command flutter -ErrorAction SilentlyContinue) { "flutter" } elseif (Test-Path "C:\flutter\bin\flutter.bat") { "C:\flutter\bin\flutter.bat" } else { "flutter" }

Write-Host "==> Running Cargo Test Suite across workspace..." -ForegroundColor Cyan
cargo test --workspace

Write-Host "==> Verifying Clippy Zero Warnings Gate..." -ForegroundColor Cyan
cargo clippy --workspace --all-targets -- -D warnings

Write-Host "==> Verifying Rust Formatting..." -ForegroundColor Cyan
cargo fmt --all -- --check

Write-Host "==> Verifying Flutter Analysis..." -ForegroundColor Cyan
Push-Location apps/flutter
try {
    & $FlutterCmd analyze
    Write-Host "==> Running Flutter Widget Tests..." -ForegroundColor Cyan
    & $FlutterCmd test
} finally {
    Pop-Location
}

Write-Host "==> Running Performance Benchmarks..." -ForegroundColor Cyan
cargo run --release --bin run_benchmarks

Write-Host "==> ALL QUALITY AND VERIFICATION GATES PASSED!" -ForegroundColor Green
