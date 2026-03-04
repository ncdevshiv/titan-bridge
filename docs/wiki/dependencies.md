# Titan Build & System Dependencies

The Titan MQL5/Rust Bridge requires a strict baseline of OS-level toolchains to guarantee **hermetic portability** via Dynamic CRT linking and zero-copy inter-process communication constraints.

## System Prerequisites

To successfully execute `bootstrap.ps1` and `cargo build`, the target Windows host (usually WS2019, WS2022, or Win10+) **MUST** have the following installed. The `bootstrap.ps1` script will halt and output mitigation instructions if they are missing.

### 1. The Rust Toolchain
- **Description**: The core Rust compiler suite.
- **Provider**: `rustup`
- **Installation**: https://rustup.rs/
- **Required Version**: Latest stable
- **Target**: `stable-x86_64-pc-windows-msvc`

### 2. CMake (For Eclipse Iceoryx)
- **Description**: CMake is utilized by the Rust `build.rs` to reach into `third_party/iceoryx` and invoke the C++ build instructions required to create the statically linked `RouDi` zero-copy memory daemon.
- **Requirement**: >=3.18
- **Installation**: Windows Package Manager (`winget`) and Admin Privileges are usually required to run the automated installation inside `bootstrap.ps1`.
- **Command**: `winget install -e --id Kitware.CMake`

### 3. Protocol Buffers Compiler (`protoc`)
- **Description**: Protobuf ensures Agnostic Payload schemas. The Rust build library `prost-build` invokes the system `protoc` execution path under the hood to compile `schemas/titan.proto` into generated `.rs` structures.
- **Requirement**: Vendored binary included in `protoc_bin/bin/protoc.exe` (hermetic build)
- **Alternative**: Must be accessible in `$ENV:PATH` if not using vendored version
- **Download**: Directly from Google protobuf repositories (https://github.com/protocolbuffers/protobuf/releases)

### 4. Git (For Iceoryx Clone)
- **Description**: Required to clone Iceoryx repository during bootstrap
- **Installation**: https://git-scm.com/download/win

---

## Rust Dependencies (Cargo.toml)

### Core Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `cxx` | 1.0 | Safe C++ interoperability via the CXX bridge |
| `once_cell` | 1.18 | Lazy static initialization for global singletons |

### Serialization & Data

| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` | 1.0 | Serialization framework with derive macros |
| `serde_json` | 1.0 | JSON serialization for config and logging |
| `prost` | 0.13 | Protocol Buffers encoding/decoding |
| `bytes` | 1 | Byte buffer utilities for Protobuf |

### Time & Integrity

| Crate | Version | Purpose |
|-------|---------|---------|
| `chrono` | 0.4 | Date/time handling for structured logging |
| `crc32fast` | 1.3 | Hardware-accelerated CRC32 checksum validation |
| `uuid` | 1.8 (v4) | Session UUID generation |

### Build Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `cxx-build` | 1.0 | CXX bridge compilation in build.rs |
| `cmake` | 0.1 | CMake integration for Iceoryx build |
| `md5` | 0.7 | Checksum utilities |
| `prost-build` | 0.13 | Protobuf code generation at build time |

---

## Iceoryx Version

**Iceoryx**: v2.95.8

**Location**: `third_party/iceoryx/`

**Build Output**:
- Static libraries linked: `iceoryx_posh`, `iceoryx_posh_roudi`, `iceoryx_hoofs`, `iceoryx_platform`
- Include path: `third_party/iceoryx/install/include/iceoryx/v2.95.8`
- Library path: `third_party/iceoryx/install/lib`

**System Library Dependencies**:
- `advapi32` (Windows system library for registry/security)

---

## Cargo Configuration

The `.cargo/config.toml` specifies dynamic CRT linking:

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=-crt-static"]
```

This ensures compatibility with the MSVC C++ runtime used by Iceoryx.

---

## Full Dependency Tree Summary

```
Titan Bridge
├── Rust 2021 Edition
│   ├── cxx 1.0 (C++ FFI)
│   ├── once_cell 1.18 (lazy statics)
│   ├── serde 1.0 + serde_json 1.0 (serialization)
│   ├── chrono 0.4 (time)
│   ├── crc32fast 1.3 (checksums)
│   ├── prost 0.13 + bytes 1 (protobuf)
│   └── uuid 1.8 (UUID generation)
├── Build Tools
│   ├── cmake >=3.18
│   ├── protoc (vendored)
│   └── git
└── Iceoryx v2.95.8 (C++)
    ├── iceoryx_posh (publish/subscribe)
    ├── iceoryx_posh_roudi (daemon)
    ├── iceoryx_hoofs (utilities)
    └── iceoryx_platform (platform abstraction)
```

---

> If these dependencies are successfully installed, running `bootstrap.ps1` fetches Iceoryx natively and configures the MSVC linkage environment to match C Runtime profiles dynamically through Cargo settings (`.cargo/config.toml`).
