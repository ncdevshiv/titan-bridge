@echo off
cd /d f:\market-rust
"C:\Program Files\Git\bin\git.exe" config user.name "Shiv"
"C:\Program Files\Git\bin\git.exe" config user.email "ncdevshiv@gmail.com"
"C:\Program Files\Git\bin\git.exe" add .
"C:\Program Files\Git\bin\git.exe" commit -m "Initial commit: Titan Protocol Bridge - Open Source Release

- 44 comprehensive tests passing
- Real C++ implementations (no stubs)
- CRC32 validation
- Circuit breakers
- Zero-copy Iceoryx integration
- JSON structured logging

Created by: Shiv (ncdevshiv@gmail.com)
Support: https://buymeacoffee.com/ncdevshiv"
echo Setup complete!
