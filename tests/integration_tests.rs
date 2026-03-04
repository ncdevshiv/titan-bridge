use titan_bridge::pb;
use titan_bridge::circuit_breakers::{MaxSlippageGuard, DataCircuitBreaker};
use titan_bridge::clock_sync::{ClockSyncPacket, TripleClock};
use titan_bridge::protocol::{BinaryTick, ProtocolNormalizer};
use prost::Message;
use std::time::Duration;
use std::thread;

// =============================================================================
// Protobuf Encoding/Decoding Tests
// =============================================================================

#[test]
fn test_protobuf_tick_encoding() {
    let pb_tick = pb::TickStream {
        symbol_id: 1,
        timestamp_ms: 1700000000000,
        bid: 1.10500,
        ask: 1.10505,
        last: 1.10500,
        volume: 100,
        sequence_id: 5,
        crc32: 123456789,
    };
    
    let mut buf = Vec::new();
    pb_tick.encode(&mut buf).expect("Failed Protobuf Encoding (Tick)");
    
    let decoded = pb::TickStream::decode(&buf[..]).expect("Failed Protobuf Decoding (Tick)");
    assert_eq!(decoded.symbol_id, 1);
    assert_eq!(decoded.sequence_id, 5);
    assert_eq!(decoded.bid, 1.10500);
    assert_eq!(decoded.ask, 1.10505);
    assert_eq!(decoded.crc32, 123456789);
}

#[test]
fn test_protobuf_dom_encoding() {
    let mut bids = Vec::new();
    bids.push(pb::DomLevel { price: 1.10, volume: 10 });
    
    let mut asks = Vec::new();
    asks.push(pb::DomLevel { price: 1.11, volume: 20 });
    
    let pb_dom = pb::DepthOfMarket {
        symbol_id: 2,
        timestamp_ms: 1700000000000,
        bids,
        asks,
    };
    
    let mut buf = Vec::new();
    pb_dom.encode(&mut buf).expect("Failed Protobuf Encoding (DOM)");
    
    let decoded = pb::DepthOfMarket::decode(&buf[..]).expect("Failed Protobuf Decoding (DOM)");
    assert_eq!(decoded.symbol_id, 2);
    assert_eq!(decoded.asks[0].volume, 20);
    assert_eq!(decoded.bids[0].price, 1.10);
}

#[test]
fn test_protobuf_symbol_metadata_encoding() {
    let pb_meta = pb::SymbolMetadata {
        symbol_id: 3,
        contract_size: 100000.0,
        point_value: 0.00001,
        digits: 5,
        execution_mode: 0,
    };
    
    let mut buf = Vec::new();
    pb_meta.encode(&mut buf).expect("Failed Protobuf Encoding (Metadata)");
    
    let decoded = pb::SymbolMetadata::decode(&buf[..]).expect("Failed Protobuf Decoding (Metadata)");
    assert_eq!(decoded.symbol_id, 3);
    assert_eq!(decoded.contract_size, 100000.0);
    assert_eq!(decoded.digits, 5);
}

#[test]
fn test_protobuf_empty_dom() {
    let pb_dom = pb::DepthOfMarket {
        symbol_id: 4,
        timestamp_ms: 1700000000000,
        bids: Vec::new(),
        asks: Vec::new(),
    };
    
    let mut buf = Vec::new();
    pb_dom.encode(&mut buf).expect("Failed to encode empty DOM");
    
    let decoded = pb::DepthOfMarket::decode(&buf[..]).expect("Failed to decode empty DOM");
    assert_eq!(decoded.symbol_id, 4);
    assert!(decoded.bids.is_empty());
    assert!(decoded.asks.is_empty());
}

#[test]
fn test_protobuf_multiple_dom_levels() {
    let bids: Vec<_> = (1..=5).map(|i| pb::DomLevel { 
        price: 1.10000 - (i as f64 * 0.00010), 
        volume: i as u64 * 100 
    }).collect();
    
    let asks: Vec<_> = (1..=5).map(|i| pb::DomLevel { 
        price: 1.10010 + (i as f64 * 0.00010), 
        volume: i as u64 * 100 
    }).collect();
    
    let pb_dom = pb::DepthOfMarket {
        symbol_id: 5,
        timestamp_ms: 1700000000000,
        bids,
        asks,
    };
    
    let mut buf = Vec::new();
    pb_dom.encode(&mut buf).expect("Failed to encode multi-level DOM");
    
    let decoded = pb::DepthOfMarket::decode(&buf[..]).expect("Failed to decode multi-level DOM");
    assert_eq!(decoded.bids.len(), 5);
    assert_eq!(decoded.asks.len(), 5);
    assert_eq!(decoded.bids[0].price, 1.09990);
    assert_eq!(decoded.asks[4].price, 1.10060);
}

// =============================================================================
// End-to-End Flow Integration Tests
// =============================================================================

#[test]
fn test_protocol_normalizer_end_to_end() {
    let mut normalizer = ProtocolNormalizer::new();
    
    // Create a valid tick with proper CRC32
    let tick = create_valid_tick(1, 1000, 1.10000, 1.10005, 1);
    
    // First tick should process successfully
    let result = normalizer.process_tick(&tick);
    assert!(result.is_ok(), "First tick should process successfully");
}

#[test]
fn test_sequence_deduplication_integration() {
    let mut normalizer = ProtocolNormalizer::new();
    
    // Process first tick
    let tick1 = create_valid_tick(1, 1000, 1.10000, 1.10005, 1);
    normalizer.process_tick(&tick1).unwrap();
    
    // Process duplicate tick (same sequence) - should be silently dropped
    let tick_dup = create_valid_tick(1, 1001, 1.10001, 1.10006, 1);
    let result = normalizer.process_tick(&tick_dup);
    assert!(result.is_ok(), "Duplicate tick should return Ok (silently dropped)");
}

#[test]
fn test_gap_detection_integration() {
    let mut normalizer = ProtocolNormalizer::new();
    
    // Process first tick
    let tick1 = create_valid_tick(1, 1000, 1.10000, 1.10005, 1);
    normalizer.process_tick(&tick1).unwrap();
    
    // Process tick with gap (sequence jumps from 1 to 5)
    let tick_gap = create_valid_tick(1, 1001, 1.10001, 1.10006, 5);
    let result = normalizer.process_tick(&tick_gap);
    assert!(result.is_ok(), "Gap tick should still process");
}

#[test]
fn test_circuit_breaker_trip_and_reset_integration() {
    let mut breaker = DataCircuitBreaker::new(5, 100); // 5 TPS, 100ms trip duration
    
    // Process 5 ticks - should all pass
    for _ in 0..5 {
        assert!(breaker.allow_tick(), "Tick should be allowed before limit");
    }
    
    // 6th tick should trip the breaker
    assert!(!breaker.allow_tick(), "6th tick should trip the breaker");
    
    // Immediately after trip, more ticks should be rejected
    assert!(!breaker.allow_tick(), "Tick should be rejected while tripped");
    
    // Wait for trip duration to pass
    thread::sleep(Duration::from_millis(150));
    
    // After reset, ticks should be allowed again
    assert!(breaker.allow_tick(), "Tick should be allowed after reset");
}

#[test]
fn test_slippage_guard_integration() {
    let mut guard = MaxSlippageGuard::new(0.00010); // 10 points max slippage
    
    guard.update_prices(1.10000, 1.10010);
    
    // Buy at expected price 1.10005 (actual ask is 1.10010, slippage = 5 points) - ALLOWED
    assert!(guard.validate_execution(1.10005, true).is_ok());
    
    // Buy at expected price 1.10000 (actual ask is 1.10010, slippage = 10 points) - ALLOWED (at limit)
    assert!(guard.validate_execution(1.10000, true).is_ok());
    
    // Buy at expected price 1.09999 (actual ask is 1.10010, slippage = 11 points) - REJECTED
    assert!(guard.validate_execution(1.09999, true).is_err());
    
    // Sell at expected price 1.10010 (actual bid is 1.10000, slippage = 10 points) - ALLOWED (at limit)
    assert!(guard.validate_execution(1.10010, false).is_ok());
    
    // Sell at expected price 1.10011 (actual bid is 1.10000, slippage = 11 points) - REJECTED
    assert!(guard.validate_execution(1.10011, false).is_err());
}

#[test]
fn test_clock_sync_ema_integration() {
    let mut clock = TripleClock::new();
    
    // Initial packet
    let packet1 = ClockSyncPacket {
        time_trade_server: 10000,
        get_tick_count_64: 5000,
    };
    clock.update_offsets(&packet1);
    
    let initial_broker_offset = clock.broker_server_offset_ms;
    let initial_ptp_offset = clock.ptp_offset;
    
    // Second packet with different timing
    let packet2 = ClockSyncPacket {
        time_trade_server: 20000,
        get_tick_count_64: 10000,
    };
    clock.update_offsets(&packet2);
    
    // EMA should smooth the transition - offsets should change but not dramatically
    assert!(clock.broker_server_offset_ms != initial_broker_offset, 
            "Broker offset should change after second packet");
    assert!(clock.ptp_offset != initial_ptp_offset, 
            "PTP offset should change after second packet");
}

// =============================================================================
// Helper Functions
// =============================================================================

fn create_valid_tick(
    symbol_id: u32,
    timestamp_ms: i64,
    bid: f64,
    ask: f64,
    sequence_id: u64,
) -> BinaryTick {
    use crc32fast::Hasher;
    
    let mut tick = BinaryTick {
        symbol_id,
        timestamp_ms,
        bid,
        ask,
        last: bid,
        volume: 100,
        sequence_id,
        crc32: 0,
    };
    
    // Calculate CRC32
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
