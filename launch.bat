@echo off
setlocal

:: Titan Protocol Launch Orchestrator

echo [1/3] Starting Eclipse Iceoryx RouDi Daemon...
:: In a normal build, the executable will be found in the cmake output dir
:: Adjust the path based on your cmake generator (e.g., Release/Debug folder)
set ROUDI_EXE=.\third_party\iceoryx\build\iceoryx_posh\Release\iox-roudi.exe

if not exist "%ROUDI_EXE%" (
    echo CRITICAL: iox-roudi.exe not found. Did you run bootstrap.ps1?
    exit /b 1
)

:: Start RouDi daemon in the background
start "RouDi Zero-Copy Daemon" "%ROUDI_EXE%"

:: Wait 2 seconds for RouDi to initialize its static memory segments
timeout /t 2 /nobreak >nul

echo [2/3] Validating Titan Protocol DLL health...
:: You could run `cargo test` here as a quick smoke-check to verify the zero-copy pipeline
cargo test --release -- --nocapture

if %errorlevel% neq 0 (
    echo CRITICAL: Backend ingestion tests failed. Halting startup.
    exit /b %errorlevel%
)

echo [3/3] Titan Protocol Bridge Active.
echo You may now start the MetaTrader 5 Terminal.
echo Leave this terminal open to monitor the RouDi daemon.
pause
