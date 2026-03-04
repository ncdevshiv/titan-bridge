# Titan Protocol - MetaTrader 5 DMA Bridge

[![License](https://img.shields.io/badge/license-Personal%20Use%20Only-blue.svg)](LICENSE)
[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-Support-yellow.svg)](https://buymeacoffee.com/ncdevshiv)

**Created by:** Shiv ([ncdevshiv@gmail.com](mailto:ncdevshiv@gmail.com))  
**Support:** ☕ [Buy Me a Coffee](https://buymeacoffee.com/ncdevshiv)

---

The **Titan Protocol Bridge** is an Ultra-Low Latency, Zero-Copy Direct Memory Access (DMA) Gateway designed to decouple MetaTrader 5 (MT5) from tight database/frontend constraints. It streams high-frequency tick data, Level 2 Deep Order Books (DOM), and Trade Metadata in a strictly normalized, database-agnostic format via **Eclipse Iceoryx**.

> 💖 **Support This Project**  
> If you find this project useful, please consider [buying me a coffee](https://buymeacoffee.com/ncdevshiv)! Your support helps maintain and improve this open source project.

This guarantees maximum protocol neutrality allowing any consumer (C++, Rust, Go, Python, Svelte via WebSockets, DuckDB, ClickHouse) to attach and receive payloads flawlessly without parsing text strings or writing to disks.

## Build Status

| Component | Status |
|-----------|--------|
| Rust/C++ FFI (CXX Bridge) | ![Working](https://img.shields.io/badge/status-working-success) |
| Iceoryx Zero-Copy Publishing | ![Working](https://img.shields.io/badge/status-working-success) |
| CRC32 Validation | ![Working](https://img.shields.io/badge/status-working-success) |
| Sequence Deduplication | ![Working](https://img.shields.io/badge/status-working-success) |
| Data Circuit Breaker | ![Working](https://img.shields.io/badge/status-working-success) |
| Max Slippage Guard | ![Working](https://img.shields.io/badge/status-working-success) |
| JSON Structured Logging | ![Working](https://img.shields.io/badge/status-working-success) |
| Release DLL Build | ![Working](https://img.shields.io/badge/status-working-success) |

## Test Coverage

**44 comprehensive tests** covering:
- **Protocol Module**: CRC32 validation, sequence deduplication, gap detection, multi-symbol tracking
- **Circuit Breakers**: TPS limiting, trip/reset cycles, window rollover
- **Max Slippage Guard**: Buy/sell validation, price updates, limit enforcement
- **Clock Sync**: Triple-clock initialization, EMA smoothing, multi-update handling
- **Configuration**: JSON loading, error handling for missing/malformed configs
- **Error Handling**: Error code validation, system error creation, context handling
- **Protobuf Integration**: Tick, DOM, and metadata encoding/decoding
- **End-to-End**: Full integration tests across all modules

Run tests with:
```sh
cargo test
```

## Architecture Highlights

- **100% Zero-Copy Streaming**: Relies on Iceoryx `iox::popo::Publisher` bypassing standard kernel OS socket and file IO overheads.
- **Rust/C++ FFI via CXX Bridge**: Seamless integration between Rust business logic and Iceoryx C++ APIs.
- **Fail Loudly Mandate**: No silent failures exist. Any missing Iceoryx routers, structural violations, or internal thread poisoning immediately halts processes safely with structured panics mapped to `ErrorCode` enumerations.
- **Protobuf First**: Messages crossing the boundary are encoded in Google Protobuf arrays representing true architectural standardization (`schemas/titan.proto`).
- **Configurable Integrity Guards**: Circuit-breakers strictly isolate downstream consumers from bad upstream burst profiles (`DataCircuitBreaker`) and malicious pricing deviations (`MaxSlippageGuard`).
- **CRC32 Validation**: All ticks undergo integrity verification via `crc32fast` to detect corrupt DMA transfers.
- **Sequence Deduplication**: Automatic ghost tick filtering and gap detection for reliable streaming.
- **JSON Structured Logging**: All events logged in structured JSON format with timestamps, modules, and error codes.

## What Actually Works

1. **Rust/C++ FFI via CXX Bridge**: Full integration with Iceoryx C++ APIs through the `shared_memory.rs` bridge.
2. **Iceoryx Zero-Copy Publishing**: Tick, book, metadata, and command streams via shared memory.
3. **CRC32 Validation on All Ticks**: Every tick validated for integrity before processing.
4. **Sequence Deduplication and Gap Detection**: Prevents ghost ticks and detects network gaps.
5. **DataCircuitBreaker with TPS Limiting**: Configurable rate limiting with automatic trip/reset.
6. **MaxSlippageGuard for Execution Validation**: Prevents fills beyond acceptable slippage thresholds.
7. **JSON Structured Logging**: Comprehensive logging to stdout and `titan_bridge.log`.
8. **44 Comprehensive Tests**: Full unit and integration test coverage.
9. **Release Build Produces Working DLL**: `target/release/titan_bridge.dll` ready for MT5 integration.

## Key Dependencies for Build

The Bridge operates fundamentally via **C++ bindings via CXX bridged through Rust**.

### Rust Dependencies (from Cargo.toml)
- **cxx** = "1.0" - C++ interoperability
- **once_cell** = "1.18" - Lazy static initialization
- **serde** = "1.0" - Serialization framework
- **serde_json** = "1.0" - JSON serialization
- **chrono** = "0.4" - Date/time handling
- **crc32fast** = "1.3" - CRC32 checksum validation
- **prost** = "0.13" - Protobuf encoding/decoding
- **bytes** = "1" - Byte buffer utilities
- **uuid** = "1.8" - Session UUID generation

### System Prerequisites
For Windows:
- **Rust Toolchain**: Must be explicitly set to standard Windows ABI `stable-x86_64-pc-windows-msvc`.
- **Cargo Configuration**: Must include dynamic linking to C Runtime definitions (CRT):
```toml
# .cargo/config.toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=-crt-static"]
```
- **CMake**: >=3.18 (for building Iceoryx)
- **Protoc**: Vendored hermetic Google Protobuf Compiler (included in `protoc_bin/`)

### Iceoryx Version
- **Iceoryx**: v2.95.8 (located in `third_party/iceoryx`)

## Setup and Bootstrapping

Run the Windows PowerShell `bootstrap.ps1` script to install all components autonomously:

```powershell
.\bootstrap.ps1
```

This executes:
1. Verification/installation of the Rust toolchain via `rustup`
2. Verification/installation of CMake via `winget`
3. Cloning and building Eclipse Iceoryx RouDi daemon from `third_party/iceoryx`
4. Building the Titan Bridge Rust DLL with dynamic CRT linking

### Manual Build (if bootstrap fails)

```sh
# Ensure CMake is installed and in PATH
# Build Iceoryx manually
cd third_party/iceoryx
cmake -B build -S iceoryx_meta -DBUILD_STRICT=OFF
cmake --build build --config Release
cd ../..

# Build the Rust DLL
cargo build --release
```

## Usage

The `titan_bridge.dll` produced in `target/release/` is seamlessly loaded into `ea/HighPerformanceBridgeEA.mq5`.

To run the full simulation locally or execute tests:
```sh
cargo clean
set CARGO_TARGET_DIR=target_custom
cargo test
cargo build --release
```

## Module Structure

```
src/
├── lib.rs              # Main DLL entry points and FFI interface
├── errors.rs           # ErrorCode enum and SystemError struct
├── config.rs           # SystemConfig JSON loading
├── protocol.rs         # BinaryTick, ProtocolNormalizer, CRC32 validation
├── circuit_breakers.rs # DataCircuitBreaker and MaxSlippageGuard
├── clock_sync.rs       # TripleClock and ClockSyncPacket
├── logger.rs           # JSON structured logging
├── shared_memory.rs    # CXX bridge to Iceoryx C++ APIs
├── pb/                 # Generated Protobuf types
├── titan_iceoryx.cpp   # C++ Iceoryx wrapper implementation
└── titan_iceoryx.hpp   # C++ Iceoryx wrapper header
```

## Documentation

Documentation regarding the agnostic architectures can be referenced directly in:
- [`docs/wiki/architecture.md`](docs/wiki/architecture.md) - System architecture and data flow
- [`docs/wiki/dependencies.md`](docs/wiki/dependencies.md) - Complete dependency reference
- [`docs/wiki/error_reference.md`](docs/wiki/error_reference.md) - Error codes and recovery strategies

## License

This project is licensed under a **Modified Personal Use License**.

- ✅ **Personal Use**: Free for personal, educational, and non-commercial use
- ✅ **Open Source**: Source code is available and modifiable for personal use
- ⚠️ **Commercial Use**: Requires explicit permission. [Contact me](mailto:ncdevshiv@gmail.com) for commercial licensing

See [LICENSE](LICENSE) for full terms.

---

## 💖 Support This Project

If you find this project useful, please consider supporting its development!

<a href="https://buymeacoffee.com/ncdevshiv" target="_blank">
  <img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" style="height: 60px !important;width: 217px !important;" >
</a>

### Why Support?
- 🚀 Helps fund ongoing development and new features
- 🐛 Enables faster bug fixes and improvements  
- 📖 Supports documentation and tutorial creation
- 💡 Your contribution directly impacts the project's future

**[☕ Buy Me a Coffee](https://buymeacoffee.com/ncdevshiv)**

---

## Contact

Created with ❤️ by **Shiv**

- 📧 Email: [ncdevshiv@gmail.com](mailto:ncdevshiv@gmail.com)
- ☕ Support: [Buy Me a Coffee](https://buymeacoffee.com/ncdevshiv)
- 🐙 GitHub: Repository managed by creator

For commercial licensing inquiries, bug reports, or feature requests, please reach out!

---

*If you use this project in your research or trading system, please consider [buying me a coffee](https://buymeacoffee.com/ncdevshiv) to show your support!*
