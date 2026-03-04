# Titan Architecture Deep-Dive

## 1. Module Structure

The Titan Protocol Bridge is organized into the following Rust modules:

| Module | File | Purpose |
|--------|------|---------|
| `lib` | `src/lib.rs` | DLL entry points, FFI interface, global singletons |
| `errors` | `src/errors.rs` | ErrorCode enum, SystemError struct, fail_loudly macro |
| `config` | `src/config.rs` | SystemConfig loading from JSON |
| `protocol` | `src/protocol.rs` | BinaryTick struct, ProtocolNormalizer, CRC32 validation |
| `circuit_breakers` | `src/circuit_breakers.rs` | DataCircuitBreaker (TPS limiting) and MaxSlippageGuard |
| `clock_sync` | `src/clock_sync.rs` | TripleClock and ClockSyncPacket for time synchronization |
| `logger` | `src/logger.rs` | JSON structured logging to stdout and file |
| `shared_memory` | `src/shared_memory.rs` | CXX bridge to Iceoryx C++ APIs |
| `pb` | Generated | Protobuf types from `schemas/titan.proto` |

### C++ Components

| Component | File | Purpose |
|-----------|------|---------|
| Iceoryx Wrapper | `src/titan_iceoryx.cpp` | C++ implementation for Iceoryx publisher/subscriber |
| Iceoryx Header | `src/titan_iceoryx.hpp` | C++ header with Iceoryx v2.95.8 bindings |

## 2. Zero-Copy IPC with Eclipse Iceoryx

The Titan Protocol completely bypasses ZMQ and TCP network sockets for ingest. The `RouDi` daemon allocates fixed blocks of RAM on startup. When MT5 produces a tick, it writes *directly* to this pre-allocated RAM through the Rust DLL (`bridge_push_tick`). There is absolutely no JSON parsing overhead, string allocation, or garbage collection in the hot path.

Downstream consumers (`iox::subscriber`) are simply handed a pointer to the memory region.

### Iceoryx Integration Details

**Version**: Iceoryx v2.95.8 (located in `third_party/iceoryx`)

**Build Process**:
1. CMake (>=3.18) builds Iceoryx from `third_party/iceoryx`
2. Static libraries linked: `iceoryx_posh`, `iceoryx_posh_roudi`, `iceoryx_hoofs`, `iceoryx_platform`
3. CXX bridge in `shared_memory.rs` provides Rust bindings to C++ Iceoryx APIs
4. Include path: `third_party/iceoryx/install/include/iceoryx/v2.95.8`

**Topic Structure**:
- Service: "Titan"
- Instance: "MarketData"  
- Topics: "TickStream", "DepthOfMarket", "SymbolMetadata", "CommandChannel"

> **CRITICAL CONFIGURATION**: Ensure `bootstrap.ps1` successfully compiles RouDi locally, as we depend on the dynamic `.cargo/config.toml` specification `target-feature=-crt-static` for `/MD` native loading on MSVC.

## 3. FFI Boundary (Rust/C++ via CXX)

The FFI boundary uses the CXX crate for safe C++/Rust interop:

**Rust Side** (`shared_memory.rs`):
- Defines `SharedMemoryContext` struct with CXX bridge macros
- Wraps C++ Iceoryx publisher/subscriber lifecycle

**C++ Side** (`titan_iceoryx.cpp`):
- Implements `TitanIceoryxPublisher` class
- Uses `iox::popo::Publisher` for zero-copy publishing
- Publishes to topics: TickStream, DepthOfMarket, SymbolMetadata

**MQL5 FFI** (`lib.rs`):
- `#[no_mangle]` extern "C" functions for MT5 DLL loading
- `bridge_init()` - Initializes all lazy statics
- `bridge_push_tick()` - Receives ticks from MT5
- `bridge_push_dom()` - Receives DOM updates from MT5
- `bridge_push_metadata()` - Receives metadata from MT5
- `bridge_clock_sync()` - Receives clock sync packets

## 4. Data Flow

```
MT5 Terminal (MQL5)
    |
    | C-struct via DLL
    v
Titan Bridge DLL (Rust FFI)
    |
    |---> CRC32 Validation (protocol.rs)
    |---> Sequence Deduplication (protocol.rs)
    |---> Circuit Breaker Check (circuit_breakers.rs)
    |---> Slippage Validation (circuit_breakers.rs)
    |
    | Protobuf encoding
    v
CXX Bridge (shared_memory.rs)
    |
    | C++ call
    v
Iceoryx Publisher (titan_iceoryx.cpp)
    |
    | Zero-copy shared memory
    v
RouDi Daemon
    |
    | Zero-copy shared memory
    v
Downstream Consumers (C++, Rust, Python, etc.)
```

## 5. Triple-Clock Synchronization (`src/clock_sync.rs`)

In high-frequency trading context, the Broker server time is inherently disconnected from the local OS execution time, which is further disconnected from Rust Thread context gaps.

To combat jitter, `ClockSyncPacket` uses:
- `time_trade_server` (Broker OS ms)
- `GetTickCount64()` (Windows Uptime ms)

The EA broadcasts these every 5 seconds. The Rust `TripleClock` singleton reads the local OS `Instant::now()` and calculates the offset. A smoothing algorithm applies an Exponential Moving Average (alpha 0.1) to adjust timestamp latency without snapping the clock out of bounds and ruining time-series chronologies.

## 6. The Data Circuit Breaker (`src/circuit_breakers.rs`)

MT5 terminals are notorious for dumping monolithic blocks of history data upon initial connection, flooding IPC architectures.

To prevent buffer overflow, `DataCircuitBreaker` dynamically monitors the ingress rate based on a `max_tps` constraint. If `tick_count > max_tps` within a 1-second `Instant` window, the stream is halted.

Instead of crashing the program, the Circuit Breaker trips, isolates the ingress, logs a critical visual (via `log_critical_error`), and implements a cooldown phase before resuming tick ingestion. This ensures the zero-copy buffer never saturates past 90% utilization.

### Circuit Breaker States

| State | Description |
|-------|-------------|
| Normal | Ticks passing through, counting against window |
| Tripped | TPS exceeded, blocking all ticks |
| Cooldown | Waiting for `trip_duration` to elapse |
| Reset | Window cleared, resuming normal operation |

## 7. Max Slippage Guard (`src/circuit_breakers.rs`)

The `MaxSlippageGuard` prevents execution at prices too far from current market:

- Tracks last known bid/ask prices
- Validates execution prices against slippage threshold
- Returns `ERR_202_MAX_SLIPPAGE_EXCEEDED` if threshold exceeded
- Configurable via `max_slippage_points` in `config/system.json`

## 8. Protocol Normalization (`src/protocol.rs`)

The `ProtocolNormalizer` handles:

### CRC32 Validation
- Every tick verified using `crc32fast`
- Returns `ERR_101_CRC_VALIDATION_FAILED` on mismatch
- Protects against corrupt DMA transfers

### Sequence Management
- Tracks `last_processed_sequence` per symbol
- Drops duplicates (ghost tick filtering)
- Detects gaps (logs warning for potential missed data)
- Returns `ERR_102_SEQUENCE_DUPLICATION_DETECTED` context

## 9. Configuration System (`src/config.rs`)

Configuration loaded from `config/system.json`:

```json
{
    "max_tps": 1000,
    "trip_duration_ms": 5000,
    "max_slippage_points": 0.0010,
    "iceoryx_threshold_percent": 90.0,
    "app_name": "TitanBridge"
}
```

Loading failures result in `ERR_004_CONFIG_FILE_ERROR`.

## 10. Global Singletons (`src/lib.rs`)

All state managed via `once_cell::sync::Lazy`:

```rust
static CONFIG: Lazy<SystemConfig> = ...
static NORMALIZER: Lazy<Mutex<ProtocolNormalizer>> = ...
static CIRCUIT_BREAKER: Lazy<Mutex<DataCircuitBreaker>> = ...
static MAX_SLIPPAGE: Lazy<Mutex<MaxSlippageGuard>> = ...
static CLOCK_SYNC: Lazy<Mutex<TripleClock>> = ...
static MEMORY_POOL: Lazy<Mutex<SharedMemoryContext>> = ...
```

Initialized on first access or explicitly in `bridge_init()`.

## 11. Agnostic State Determinism

The Titan Protocol acts purely as an ingestor/publisher framework. Virtual SL/TP levels are never maintained within MT5 strings or object states natively in this bridge layer. If MT5 crashes, the incoming Backend Engineer must configure downstream subscribers (e.g. Python, ClickHouse) to handle Session UUID discrepancy invalidation to prevent stale executions firing blindly against old ticks.

The Session UUID is generated on each EA startup and broadcast via SymbolMetadata heartbeats.
