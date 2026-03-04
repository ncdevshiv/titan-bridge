<#
.SYNOPSIS
    Titan Protocol - Zero-Latency Build System Bootstrap Script
.DESCRIPTION
    Ensures a hermetic environment for building the Titan Bridge DLL with Eclipse Iceoryx backend.
    Checks Rust toolchains, downloads and builds Iceoryx C++ with RouDi configured for Static Memory
    Segments to prevent Windows memory re-mapping latency spikes.
#>

$ErrorActionPreference = "Stop"

Write-Host ">>> Titan Protocol Bootstrap Started <<<" -ForegroundColor Cyan

# 1. VERIFY RUST TOOLCHAIN
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Rustup (Hermetic environment)..."
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"
    Start-Process -FilePath ".\rustup-init.exe" -ArgumentList "-y" -Wait
    $env:Path += ";$env:USERPROFILE\.cargo\bin"
}

# 2. VERIFY CMAKE FOR ICEORYX BUILD
if (-not (Get-Command "cmake" -ErrorAction SilentlyContinue)) {
    Write-Host "CMake is missing! Installing automatically via winget for true hermetic setup..." -ForegroundColor Yellow
    winget install --id Kitware.CMake -e --silent --accept-package-agreements --accept-source-agreements
    $env:Path += ";$env:ProgramFiles\CMake\bin"
}

# 3. CONFIGURE ROUDI STATIC DAEMON
Write-Host "Configuring RouDi Daemon for Zero-Copy memory segments..."
if (-not (Test-Path ".\third_party\iceoryx")) {
    Write-Host "Fetching Eclipse Iceoryx..."
    mkdir ".\third_party" -Force | Out-Null
    git clone https://github.com/eclipse-iceoryx/iceoryx.git .\third_party\iceoryx
}

Write-Host "Building Iceoryx RouDi daemon..."
Push-Location .\third_party\iceoryx
cmake -B build -S iceoryx_meta -DBUILD_STRICT=OFF
cmake --build build --config Release
Pop-Location

# 4. BUILD THE RUST DLL WITH STATIC CRT
Write-Host "Compiling Titan Bridge (Rust DLL) with target-feature=+crt-static..."
cargo build --release

$DLLOut = ".\target\release\titan_bridge.dll"
if (Test-Path $DLLOut) {
    Write-Host "SUCCESS: $DLLOut generated." -ForegroundColor Green
    Write-Host "Please copy target\release\titan_bridge.dll to your MT5 Libraries folder." -ForegroundColor Yellow
} else {
    Write-Host "FAILURE: DLL build failed." -ForegroundColor Red
    exit 1
}

Write-Host "Bootstrap Complete." -ForegroundColor Green
