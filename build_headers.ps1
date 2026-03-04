$ErrorActionPreference = "Stop"

Write-Host "Fetching hermetic portable CMake..."
Invoke-WebRequest -Uri "https://github.com/Kitware/CMake/releases/download/v3.28.3/cmake-3.28.3-windows-x86_64.zip" -OutFile "cmake.zip"
Expand-Archive -Path "cmake.zip" -DestinationPath "cmake_bin" -Force

$env:Path += ";$PWD\cmake_bin\cmake-3.28.3-windows-x86_64\bin"

Write-Host "Bootstrapping Iceoryx headers..."
cd third_party\iceoryx
cmake -B build -S iceoryx_meta -DBUILD_STRICT=OFF
cmake --build build --config Release
cd ..\..

Write-Host "Iceoryx Platform headers successfully generated."
