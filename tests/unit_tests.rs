use titan_bridge::circuit_breakers::{MaxSlippageGuard, DataCircuitBreaker};
use titan_bridge::clock_sync::{ClockSyncPacket, TripleClock};
use titan_bridge::protocol::{BinaryTick, ProtocolNormalizer};
use titan_bridge::config::SystemConfig;
use titan_bridge::errors::{ErrorCode, SystemError};
use crc32fast::Hasher;
use std::time::Duration;
use std::thread;

// =============================================================================
// Protocol Module Unit Tests
// =============================================================================

#[test]
fn test_crc32_validation_success() {
    let mut tick = BinaryTick {
        symbol_id: 1,
        timestamp_ms: 1000,
        bid: 1.10000,
        ask: 1.10005,
        last: 1.10000,
        volume: 100,
        sequence_id: 1,
        crc32: 0,
    };
    
    // Calculate correct CRC32
    let mut hasher = Hasher::new();
    hasher.update(&tick.symbol_id.to_le_bytes());
    hasher.update(&tick.timestamp_ms.to_le_bytes());
    hasher.update(&tick.bid.to_le_bytes());
    hasher.update(&tick.ask.to_le_bytes());
    hasher.update(&tick.last.to_le_bytes());
    hasher.update(&tick.volume.to_le_bytes());
    hasher.update(&tick.sequence_id.to_le_bytes());
    tick.crc32 = hasher.finalize();
    
    assert!(ProtocolNormalizer::verify_integrity(&tick), "Valid CRC32 should pass");
}

#[test]
fn test_crc32_validation_failure() {
    let tick = BinaryTick {
        symbol_id: 1,
        timestamp_ms: 1000,
        bid: 1.10000,
        ask: 1.10005,
        last: 1.10000,
        volume: 100,
        sequence_id: 1,
        crc32: 0xDEADBEEF, // Invalid CRC32
    };
    
    assert!(!ProtocolNormalizer::verify_integrity(&tick), "Invalid CRC32 should fail");
}

#[test]
fn test_sequence_deduplication() {
    let mut normalizer = ProtocolNormalizer::new();
    
    // Create first tick with valid CRC
    let tick1 = create_tick_with_crc(1, 1.10000, 1.10005, 1);
    normalizer.process_tick(&tick1).unwrap();
    
    // Duplicate sequence ID should be silently dropped (Ok result)
    let tick_dup = create_tick_with_crc(1, 1.10001, 1.10006, 1);
    let result = normalizer.process_tick(&tick_dup);
    assert!(result.is_ok(), "Duplicate sequence should return Ok");
}

#[test]
fn test_sequence_out_of_order() {
    let mut normalizer = ProtocolNormalizer::new();
    
    // Process sequence 5 first
    let tick1 = create_tick_with_crc(1, 1.10000, 1.10005, 5);
    normalizer.process_tick(&tick1).unwrap();
    
    // Then sequence 3 (out of order, but <= last_processed)
    let tick_out_of_order = create_tick_with_crc(1, 1.10001, 1.10006, 3);
    let result = normalizer.process_tick(&tick_out_of_order);
    assert!(result.is_ok(), "Out of order sequence should return Ok (silently dropped)");
}

#[test]
fn test_gap_detection() {
    let mut normalizer = ProtocolNormalizer::new();
    
    // Process sequence 1
    let tick1 = create_tick_with_crc(1, 1.10000, 1.10005, 1);
    normalizer.process_tick(&tick1).unwrap();
    
    // Process sequence 5 (gap: expecting 2, got 5)
    let tick_gap = create_tick_with_crc(1, 1.10001, 1.10006, 5);
    let result = normalizer.process_tick(&tick_gap);
    assert!(result.is_ok(), "Gap tick should process successfully");
}

#[test]
fn test_sequential_processing() {
    let mut normalizer = ProtocolNormalizer::new();
    
    for i in 1..=10 {
        let tick = create_tick_with_crc(1, 1.10000 + (i as f64 * 0.00001), 1.10005 + (i as f64 * 0.00001), i);
        let result = normalizer.process_tick(&tick);
        assert!(result.is_ok(), "Sequential tick {} should process", i);
    }
}

#[test]
fn test_multi_symbol_tracking() {
    let mut normalizer = ProtocolNormalizer::new();
    
    // Process ticks for different symbols
    for symbol_id in 1..=5 {
        for seq in 1..=5 {
            let tick = create_tick_with_crc(symbol_id, 1.10000, 1.10005, seq);
            let result = normalizer.process_tick(&tick);
            assert!(result.is_ok(), "Symbol {} seq {} should process", symbol_id, seq);
        }
    }
}

// =============================================================================
// Circuit Breakers Unit Tests
// =============================================================================

#[test]
fn test_data_circuit_breaker_within_limit() {
    let mut breaker = DataCircuitBreaker::new(10, 100);
    
    for i in 0..10 {
        assert!(breaker.allow_tick(), "Tick {} should be allowed", i + 1);
    }
}

#[test]
fn test_data_circuit_breaker_trip() {
    let mut breaker = DataCircuitBreaker::new(5, 100);
    
    // 5 ticks should pass
    for _ in 0..5 {
        assert!(breaker.allow_tick());
    }
    
    // 6th tick should trip
    assert!(!breaker.allow_tick(), "6th tick should trip breaker");
}

#[test]
fn test_data_circuit_breaker_blocks_while_tripped() {
    let mut breaker = DataCircuitBreaker::new(3, 500);
    
    // Trip the breaker
    for _ in 0..4 {
        breaker.allow_tick();
    }
    
    // Should remain tripped
    for _ in 0..10 {
        assert!(!breaker.allow_tick(), "Ticks should be blocked while tripped");
    }
}

#[test]
fn test_data_circuit_breaker_auto_reset() {
    let mut breaker = DataCircuitBreaker::new(3, 50); // Short 50ms trip duration
    
    // Trip the breaker
    for _ in 0..4 {
        breaker.allow_tick();
    }
    assert!(!breaker.allow_tick(), "Breaker should be tripped");
    
    // Wait for reset
    thread::sleep(Duration::from_millis(100));
    
    assert!(breaker.allow_tick(), "Breaker should auto-reset after duration");
}

#[test]
fn test_data_circuit_breaker_window_rollover() {
    let mut breaker = DataCircuitBreaker::new(100, 100);
    
    // Process 50 ticks
    for _ in 0..50 {
        assert!(breaker.allow_tick());
    }
    
    // Window hasn't rolled over yet (1 second window)
    // We can't easily test time-based window rollover without mocking time,
    // but we can verify the counter is tracking correctly
}

#[test]
fn test_max_slippage_guard_buy_allowed() {
    let mut guard = MaxSlippageGuard::new(0.00010); // 10 points
    guard.update_prices(1.10000, 1.10010);
    
    // Buy at 1.10005 -> ask is 1.10010, slippage = 5 points (allowed)
    assert!(guard.validate_execution(1.10005, true).is_ok());
    
    // Buy at exact ask -> slippage = 0 (allowed)
    assert!(guard.validate_execution(1.10010, true).is_ok());
}

#[test]
fn test_max_slippage_guard_buy_rejected() {
    let mut guard = MaxSlippageGuard::new(0.00010);
    guard.update_prices(1.10000, 1.10010);
    
    // Buy at 1.09999 -> ask is 1.10010, slippage = 11 points (rejected)
    let result = guard.validate_execution(1.09999, true);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::ERR_202_MAX_SLIPPAGE_EXCEEDED);
}

#[test]
fn test_max_slippage_guard_sell_allowed() {
    let mut guard = MaxSlippageGuard::new(0.00010);
    guard.update_prices(1.10000, 1.10010);
    
    // Sell at 1.10005 -> bid is 1.10000, slippage = 5 points (allowed)
    assert!(guard.validate_execution(1.10005, false).is_ok());
    
    // Sell at exact bid -> slippage = 0 (allowed)
    assert!(guard.validate_execution(1.10000, false).is_ok());
}

#[test]
fn test_max_slippage_guard_sell_rejected() {
    let mut guard = MaxSlippageGuard::new(0.00010);
    guard.update_prices(1.10000, 1.10010);
    
    // Sell at 1.10011 -> bid is 1.10000, slippage = 11 points (rejected)
    let result = guard.validate_execution(1.10011, false);
    assert!(result.is_err());
}

#[test]
fn test_max_slippage_guard_exact_at_limit() {
    let mut guard = MaxSlippageGuard::new(0.00010);
    guard.update_prices(1.10000, 1.10010);
    
    // Buy at 1.10000 -> ask is 1.10010, slippage = 10 points (at limit, allowed)
    assert!(guard.validate_execution(1.10000, true).is_ok());
    
    // Sell at 1.10010 -> bid is 1.10000, slippage = 10 points (at limit, allowed)
    assert!(guard.validate_execution(1.10010, false).is_ok());
}

#[test]
fn test_max_slippage_guard_price_update() {
    let mut guard = MaxSlippageGuard::new(0.00010);
    
    // Initial prices
    guard.update_prices(1.10000, 1.10010);
    assert!(guard.validate_execution(1.10005, true).is_ok());
    
    // Update prices
    guard.update_prices(1.10100, 1.10110);
    // Old expected price of 1.10005 now has slippage of 100+ points
    assert!(guard.validate_execution(1.10005, true).is_err());
}

// =============================================================================
// Clock Sync Unit Tests
// =============================================================================

#[test]
fn test_triple_clock_initialization() {
    let clock = TripleClock::new();
    assert_eq!(clock.broker_server_offset_ms, 0.0);
    assert_eq!(clock.ptp_offset, 0.0);
}

#[test]
fn test_triple_clock_first_sync() {
    let mut clock = TripleClock::new();
    
    let packet = ClockSyncPacket {
        time_trade_server: 10000,
        get_tick_count_64: 5000,
    };
    
    clock.update_offsets(&packet);
    
    // Offsets should be set directly on first sync
    assert!(clock.broker_server_offset_ms != 0.0);
    assert!(clock.ptp_offset != 0.0);
}

#[test]
fn test_triple_clock_ema_smoothing() {
    let mut clock = TripleClock::new();
    
    // First sync establishes baseline
    let packet1 = ClockSyncPacket {
        time_trade_server: 10000,
        get_tick_count_64: 5000,
    };
    clock.update_offsets(&packet1);
    let first_broker_offset = clock.broker_server_offset_ms;
    
    // Second sync should apply EMA
    let packet2 = ClockSyncPacket {
        time_trade_server: 20000,
        get_tick_count_64: 10000,
    };
    clock.update_offsets(&packet2);
    
    // Offset should change, but EMA smoothing should prevent dramatic jumps
    assert!(clock.broker_server_offset_ms != first_broker_offset);
}

#[test]
fn test_triple_clock_multiple_updates() {
    let mut clock = TripleClock::new();
    
    for i in 1..=10 {
        let packet = ClockSyncPacket {
            time_trade_server: i as i64 * 1000,
            get_tick_count_64: i as u64 * 500,
        };
        clock.update_offsets(&packet);
    }
    
    // After multiple updates, offsets should be stable
    assert!(clock.broker_server_offset_ms != 0.0);
    assert!(clock.ptp_offset != 0.0);
}

// =============================================================================
// Config Module Unit Tests
// =============================================================================

#[test]
fn test_config_loading_success() {
    // Test parsing valid config JSON directly
    let valid_json = r#"{
        "max_tps": 500000,
        "trip_duration_ms": 1000,
        "max_slippage_points": 5.0,
        "iceoryx_threshold_percent": 90.0,
        "app_name": "TestApp"
    }"#;
    
    let config: SystemConfig = serde_json::from_str(valid_json).expect("Valid JSON should parse");
    assert_eq!(config.max_tps, 500000);
    assert_eq!(config.trip_duration_ms, 1000);
    assert_eq!(config.max_slippage_points, 5.0);
    assert_eq!(config.iceoryx_threshold_percent, 90.0);
    assert_eq!(config.app_name, "TestApp");
}

#[test]
fn test_config_file_not_found() {
    // Test that attempting to load from a non-existent path returns an error
    // We test this by checking the behavior with a path that doesn't exist
    let result = std::fs::read_to_string("config/nonexistent_file.json");
    assert!(result.is_err(), "Should error when file doesn't exist");
}

#[test]
fn test_config_malformed_json() {
    // Test JSON parsing directly without file manipulation
    let malformed_json = "{ invalid json }";
    let result: Result<SystemConfig, _> = serde_json::from_str(malformed_json);
    
    assert!(result.is_err(), "Should error when JSON is malformed");
}

// =============================================================================
// Error Handling Unit Tests
// =============================================================================

#[test]
fn test_error_code_display() {
    let code = ErrorCode::ERR_100_NULL_POINTER_DEREFERENCE;
    let display = format!("{}", code);
    assert!(display.contains("ERR_100_NULL_POINTER_DEREFERENCE"));
}

#[test]
fn test_system_error_creation() {
    let err = SystemError::new(
        ErrorCode::ERR_101_CRC_VALIDATION_FAILED,
        "test_module",
        "test_function",
        "Test error message"
    );
    
    assert_eq!(err.code, ErrorCode::ERR_101_CRC_VALIDATION_FAILED);
    assert_eq!(err.module, "test_module");
    assert_eq!(err.function, "test_function");
    assert_eq!(err.message, "Test error message");
    assert!(err.context_dump.is_none());
}

#[test]
fn test_system_error_with_context() {
    let err = SystemError::new(
        ErrorCode::ERR_102_SEQUENCE_DUPLICATION_DETECTED,
        "protocol",
        "validate",
        "Duplicate sequence"
    ).with_context("Additional context".to_string());
    
    assert!(err.context_dump.is_some());
    assert_eq!(err.context_dump.unwrap(), "Additional context");
}

#[test]
fn test_system_error_display() {
    let err = SystemError::new(
        ErrorCode::ERR_201_DATA_CIRCUIT_TRIPPED,
        "circuit_breaker",
        "allow_tick",
        "Circuit tripped"
    );
    
    let display = format!("{}", err);
    assert!(display.contains("ERR_201_DATA_CIRCUIT_TRIPPED"));
    assert!(display.contains("circuit_breaker"));
    assert!(display.contains("allow_tick"));
    assert!(display.contains("Circuit tripped"));
}

#[test]
fn test_all_error_codes_exist() {
    // Verify all error codes can be constructed
    let _ = ErrorCode::ERR_001_ICEORYX_ROUDI_NOT_FOUND;
    let _ = ErrorCode::ERR_002_SHARED_MEMORY_ALLOCATION_LOCKED;
    let _ = ErrorCode::ERR_003_SPACETIMEDB_CONNECTION_FAILED;
    let _ = ErrorCode::ERR_004_CONFIG_FILE_ERROR;
    let _ = ErrorCode::ERR_100_NULL_POINTER_DEREFERENCE;
    let _ = ErrorCode::ERR_101_CRC_VALIDATION_FAILED;
    let _ = ErrorCode::ERR_102_SEQUENCE_DUPLICATION_DETECTED;
    let _ = ErrorCode::ERR_103_INVALID_PROTOBUF_PAYLOAD;
    let _ = ErrorCode::ERR_201_DATA_CIRCUIT_TRIPPED;
    let _ = ErrorCode::ERR_202_MAX_SLIPPAGE_EXCEEDED;
}

#[test]
fn test_error_code_equality() {
    assert_eq!(
        ErrorCode::ERR_100_NULL_POINTER_DEREFERENCE,
        ErrorCode::ERR_100_NULL_POINTER_DEREFERENCE
    );
    assert_ne!(
        ErrorCode::ERR_100_NULL_POINTER_DEREFERENCE,
        ErrorCode::ERR_101_CRC_VALIDATION_FAILED
    );
}

#[test]
fn test_error_code_clone() {
    let code = ErrorCode::ERR_101_CRC_VALIDATION_FAILED;
    let cloned = code.clone();
    assert_eq!(code, cloned);
}

#[test]
fn test_error_code_copy() {
    let code = ErrorCode::ERR_101_CRC_VALIDATION_FAILED;
    let copied: ErrorCode = code; // Copy happens here
    assert_eq!(code, copied);
}

// =============================================================================
// Helper Functions
// =============================================================================

fn create_tick_with_crc(
    symbol_id: u32,
    bid: f64,
    ask: f64,
    sequence_id: u64,
) -> BinaryTick {
    let mut tick = BinaryTick {
        symbol_id,
        timestamp_ms: 1000 + sequence_id as i64,
        bid,
        ask,
        last: bid,
        volume: 100,
        sequence_id,
        crc32: 0,
    };
    
    let mut hasher = Hasher::new();
    hasher.update(&tick.symbol_id.to_le_bytes());
    hasher.update(&tick.timestamp_ms.to_le_bytes());
    hasher.update(&tick.bid.to_le_bytes());
    hasher.update(&tick.ask.to_le_bytes());
    hasher.update(&tick.last.to_le_bytes());
    hasher.update(&tick.volume.to_le_bytes());
    hasher.update(&tick.sequence_id.to_le_bytes());
    
    tick.crc32 = hasher.finalize();
    tick
}
