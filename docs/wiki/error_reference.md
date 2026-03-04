# Titan System Error Reference

This table represents the absolute source of truth for the Deterministic Error Framework inside the Titan Agnostic DMA Bridge.
All errors emit via `log_critical_error` (or `log_warn`) and, if critical, invoke a `fail_loudly` halt to prevent silent state corruption.

## Error Code Table

| Code | Label | Severity | Module | Core Root Cause | Suggested Mitigation / Fix |
|------|-------|----------|--------|-----------------|---------------------------|
| ERR_001 | ICEORYX_ROUDI_NOT_FOUND | CRITICAL | shared_memory | Active `RouDi` daemon is not reachable on the host OS via POSIX shared memory. | Ensure `bootstrap.ps1` successfully built Iceoryx and `iox-roudi` binary is actively running in a background process before starting the EA. |
| ERR_002 | SHARED_MEMORY_ALLOCATION_LOCKED | CRITICAL | lib | The `Lazy::Mutex` singleton holding the `SharedMemoryContext` or `NORMALIZER` poisoned a lock. | This indicates a fatal Rust thread panic inside the hot path. Check `titan_bridge.log` for anomalous thread terminations in prior steps. Restart DLL. |
| ERR_003 | SPACETIMEDB_CONNECTION_FAILED | CRITICAL | (reserved) | StateManager failed to resolve the remote or local SpacetimeDB table module. | Reserved for future SpacetimeDB integration. Currently not actively used in agnostic bridge mode. |
| ERR_004 | CONFIG_FILE_ERROR | CRITICAL | config | `config/system.json` is missing or contains malformed JSON. | Verify the configuration file exists at `config/system.json` and contains valid JSON with all required fields: max_tps, trip_duration_ms, max_slippage_points, iceoryx_threshold_percent, app_name. |
| ERR_100 | NULL_POINTER_DEREFERENCE | CRITICAL | lib | MQL5 `HighPerformanceBridgeEA` sent a null Struct pointer across the FFI boundary. | MQL5 memory space corruption. Restart MT5 Terminal immediately. Check EA C-struct padding alignment in `TitanProtocol.mqh`. |
| ERR_101 | CRC_VALIDATION_FAILED | CRITICAL | protocol | The calculated checksum on the Rust side failed to match the MT5 struct payload properties. | Corrupt DMA transfer. High likelihood of memory race conditions OR struct definition mismatch between `TitanProtocol.mqh` and `src/protocol.rs`. Verify field alignment and sizes match exactly. |
| ERR_102 | SEQUENCE_DUPLICATION_DETECTED | WARNING | protocol | The EA pushed a Tick sequence ID less than or equal to the last recorded ID. | Usually safe to ignore (Ghost Tick Filtering). If chronic, the EA might be executing cyclic logic errors in `OnTick()`. Check EA sequence ID generation. |
| ERR_103 | INVALID_PROTOBUF_PAYLOAD | CRITICAL | shared_memory | Prost serialization into the `schemas/titan.proto` byte-buffer failed unexpectedly. | Extremely rare runtime out-of-memory error inside Rust allocator or corrupted data structure. Check available system memory. |
| ERR_201 | DATA_CIRCUIT_TRIPPED | CRITICAL/WARNING | circuit_breakers | Market Data inbound TPS exceeded the configured `max_tps` window threshold (e.g., historical tick dumps). | The circuit breaker will pause intake, sleep for the cooldown duration (`trip_duration_ms`), and resume automatically. No manual intervention required. Adjust `max_tps` in config if legitimate burst. |
| ERR_202 | MAX_SLIPPAGE_EXCEEDED | WARNING | circuit_breakers | Virtual execute price diverged from current depth-of-market quotes by more than configured slippage limit. | Expected behavior during high-volatility news events. Order intentionally blocked to prevent lethal fills. Adjust `max_slippage_points` in config if too restrictive. |

## Error Code Ranges

| Range | Category |
|-------|----------|
| 000-099 | Initialization & Infrastructure Criticals |
| 100-199 | Protocol & MQL5 FFI Layer |
| 200-299 | Market Data & Circuit Breakers |
| 300-399 | (Reserved for Future Use) |
| 400-499 | (Reserved for Future Use) |
| 500-599 | (Reserved for External Integrations) |

## Error Code Definitions (src/errors.rs)

```rust
pub enum ErrorCode {
    // 000-099: Initialization & Infrastructure Criticals
    ERR_001_ICEORYX_ROUDI_NOT_FOUND = 1,
    ERR_002_SHARED_MEMORY_ALLOCATION_LOCKED = 2,
    ERR_003_SPACETIMEDB_CONNECTION_FAILED = 3,
    ERR_004_CONFIG_FILE_ERROR = 4,
    
    // 100-199: Protocol & MQL5 FFI Layer
    ERR_100_NULL_POINTER_DEREFERENCE = 100,
    ERR_101_CRC_VALIDATION_FAILED = 101,
    ERR_102_SEQUENCE_DUPLICATION_DETECTED = 102,
    ERR_103_INVALID_PROTOBUF_PAYLOAD = 103,
    
    // 200-299: Market Data & Circuit Breakers
    ERR_201_DATA_CIRCUIT_TRIPPED = 201,
    ERR_202_MAX_SLIPPAGE_EXCEEDED = 202,
}
```

## Recovery Strategies

### Critical Errors (Process Halt)
Critical errors invoke `fail_loudly!` which:
1. Logs the error via `log_critical_error`
2. Panics with the error message
3. Terminates the DLL process

**Recovery**: Restart the MT5 EA and investigate root cause from logs.

### Warning Errors (Logged, Continue)
Warning errors log via `log_warn` but continue operation:
- ERR_102: Duplicate sequences are silently dropped
- ERR_202: Slippage violations block specific orders but continue streaming

**Recovery**: No immediate action required. Monitor for patterns indicating configuration issues.

### Circuit Breaker (Auto-Recovery)
The Data Circuit Breaker (ERR_201) automatically:
1. Blocks ticks when TPS exceeded
2. Waits for `trip_duration_ms`
3. Resets and resumes automatically

**Recovery**: Automatic. Monitor logs for trip frequency.

## Log Output Format

All errors are logged in structured JSON:

```json
{
  "timestamp": "2024-01-15T10:30:00.000Z",
  "level": "CRITICAL",
  "module": "protocol",
  "function": "process_tick",
  "message": "CRC validation failed (corrupt DMA transfer) [ERR_101_CRC_VALIDATION_FAILED]",
  "error_code": "ERR_101_CRC_VALIDATION_FAILED"
}
```

## Testing Error Scenarios

All error codes have corresponding test coverage:

- `test_crc32_validation_failure` → ERR_101
- `test_error_code_display` → All error codes
- `test_system_error_creation` → SystemError construction
- `test_config_file_not_found` → ERR_004
- `test_config_malformed_json` → ERR_004
- `test_data_circuit_breaker_trip` → ERR_201
- `test_max_slippage_guard_buy_rejected` → ERR_202

Run tests: `cargo test`
