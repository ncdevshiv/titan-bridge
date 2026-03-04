use crate::errors::{ErrorCode, SystemError};
use prost::Message;

pub mod protocol;
pub mod circuit_breakers;
pub mod clock_sync;
pub mod logger;
pub mod shared_memory; 
pub mod errors;
pub mod config;

pub mod pb {
    // Inject generated prost payload structs
    include!(concat!(env!("OUT_DIR"), "/titan.marketdata.rs"));
}

use std::sync::Mutex;
use once_cell::sync::Lazy;

use protocol::{BinaryTick, ProtocolNormalizer};
use circuit_breakers::{MaxSlippageGuard, DataCircuitBreaker};
use clock_sync::{ClockSyncPacket, TripleClock};
use shared_memory::SharedMemoryContext;
use config::SystemConfig;

pub static CONFIG: Lazy<SystemConfig> = Lazy::new(|| {
    SystemConfig::load().unwrap_or_else(|e| fail_loudly!(e))
});

// Global Singletons for DLL state
static NORMALIZER: Lazy<Mutex<ProtocolNormalizer>> = Lazy::new(|| Mutex::new(ProtocolNormalizer::new()));
static CIRCUIT_BREAKER: Lazy<Mutex<DataCircuitBreaker>> = Lazy::new(|| Mutex::new(DataCircuitBreaker::new(CONFIG.max_tps, CONFIG.trip_duration_ms))); 
static MAX_SLIPPAGE: Lazy<Mutex<MaxSlippageGuard>> = Lazy::new(|| Mutex::new(MaxSlippageGuard::new(CONFIG.max_slippage_points))); 
static CLOCK_SYNC: Lazy<Mutex<TripleClock>> = Lazy::new(|| Mutex::new(TripleClock::new()));
static MEMORY_POOL: Lazy<Mutex<SharedMemoryContext>> = Lazy::new(|| Mutex::new(SharedMemoryContext::new(&CONFIG.app_name)));

// Structs matching MQL5 layout for FFI
#[repr(C)]
pub struct BinaryDOMLevel {
    pub price: f64,
    pub volume: u64,
}

#[repr(C)]
pub struct BinaryDOM {
    pub symbol_id: u32,
    pub timestamp_ms: i64,
    pub num_bids: u32,
    pub num_asks: u32,
    pub bids: [BinaryDOMLevel; 20],
    pub asks: [BinaryDOMLevel; 20],
    pub crc32: u32,
}

#[repr(C)]
pub struct MetadataPacket {
    pub symbol_id: u32,
    pub contract_size: f64,
    pub point_value: f64,
    pub digits: u32,
    pub execution_mode: i64,
}

#[no_mangle]
pub extern "C" fn bridge_init() -> bool {
    logger::log_info("lib", "bridge_init", "Titan Protocol Bridge initializing...");
    // Force initialization of all lazy statics to detect issues early
    let _unused = NORMALIZER.lock().unwrap_or_else(|_| {
        fail_loudly!(SystemError::new(ErrorCode::ERR_002_SHARED_MEMORY_ALLOCATION_LOCKED, "lib", "bridge_init", "NORMALIZER lock poisoned."));
    });
    let _unused = CIRCUIT_BREAKER.lock().unwrap();
    let _unused = MAX_SLIPPAGE.lock().unwrap();
    let _unused = CLOCK_SYNC.lock().unwrap();
    let _unused = MEMORY_POOL.lock().unwrap();
    
    logger::log_info("lib", "bridge_init", "Titan Protocol Bridge initialized successfully.");
    true
}

#[no_mangle]
pub extern "C" fn bridge_push_tick(tick_ptr: *const BinaryTick) -> bool {
    if tick_ptr.is_null() { 
        fail_loudly!(SystemError::new(ErrorCode::ERR_100_NULL_POINTER_DEREFERENCE, "lib", "bridge_push_tick", "Received null pointer for BinaryTick"));
    }

    let tick = unsafe { &*tick_ptr };

    if !CIRCUIT_BREAKER.lock().unwrap().allow_tick() {
        return false;
    }

    match NORMALIZER.lock().unwrap().process_tick(tick) {
        Ok(_) => {},
        Err(e) => {
            fail_loudly!(e);
        }
    }

    if let Err(e) = MAX_SLIPPAGE.lock().unwrap().validate_execution(tick.bid, true) {
        crate::logger::log_critical_error(&e);
    }
    MAX_SLIPPAGE.lock().unwrap().update_prices(tick.bid, tick.ask);

    let pb_tick = pb::TickStream {
        symbol_id: tick.symbol_id,
        timestamp_ms: tick.timestamp_ms,
        bid: tick.bid,
        ask: tick.ask,
        last: tick.last,
        volume: tick.volume,
        sequence_id: tick.sequence_id,
        crc32: tick.crc32,
    };
    
    let mut buf = Vec::new();
    if let Err(e) = pb_tick.encode(&mut buf) {
        fail_loudly!(SystemError::new(ErrorCode::ERR_103_INVALID_PROTOBUF_PAYLOAD, "lib", "bridge_push_tick", format!("Protobuf encode fail: {}", e)));
    }
    
    MEMORY_POOL.lock().unwrap().publish_tick_bytes(&buf)
}

#[no_mangle]
pub extern "C" fn bridge_push_book(dom_ptr: *const BinaryDOM) -> bool {
    if dom_ptr.is_null() {
        fail_loudly!(SystemError::new(ErrorCode::ERR_100_NULL_POINTER_DEREFERENCE, "lib", "bridge_push_book", "Received null pointer for BinaryDOM"));
    }
    
    let dom = unsafe { &*dom_ptr };
    
    let mut bids = Vec::new();
    for i in 0..(dom.num_bids as usize).min(20) {
        bids.push(pb::DomLevel {
            price: dom.bids[i].price,
            volume: dom.bids[i].volume,
        });
    }
    
    let mut asks = Vec::new();
    for i in 0..(dom.num_asks as usize).min(20) {
        asks.push(pb::DomLevel {
            price: dom.asks[i].price,
            volume: dom.asks[i].volume,
        });
    }

    let pb_dom = pb::DepthOfMarket {
        symbol_id: dom.symbol_id,
        timestamp_ms: dom.timestamp_ms,
        bids,
        asks,
    };
    
    let mut buf = Vec::new();
    if let Err(e) = pb_dom.encode(&mut buf) {
        fail_loudly!(SystemError::new(ErrorCode::ERR_103_INVALID_PROTOBUF_PAYLOAD, "lib", "bridge_push_book", format!("Protobuf encode fail: {}", e)));
    }
    
    MEMORY_POOL.lock().unwrap().publish_book_bytes(&buf)
}

#[no_mangle]
pub extern "C" fn bridge_push_metadata(meta_ptr: *const MetadataPacket) -> bool {
    if meta_ptr.is_null() {
        fail_loudly!(SystemError::new(ErrorCode::ERR_100_NULL_POINTER_DEREFERENCE, "lib", "bridge_push_metadata", "Received null pointer for MetadataPacket"));
    }
    
    let meta = unsafe { &*meta_ptr };
    
    let pb_meta = pb::SymbolMetadata {
        symbol_id: meta.symbol_id,
        contract_size: meta.contract_size,
        point_value: meta.point_value,
        digits: meta.digits,
        execution_mode: meta.execution_mode,
    };
    
    let mut buf = Vec::new();
    if let Err(e) = pb_meta.encode(&mut buf) {
        fail_loudly!(SystemError::new(ErrorCode::ERR_103_INVALID_PROTOBUF_PAYLOAD, "lib", "bridge_push_metadata", format!("Protobuf encode fail: {}", e)));
    }

    MEMORY_POOL.lock().unwrap().publish_metadata_bytes(&buf)
}

#[no_mangle]
pub extern "C" fn bridge_sync_clock(packet_ptr: *const ClockSyncPacket) -> bool {
    if packet_ptr.is_null() { return false; }

    let packet = unsafe { &*packet_ptr };
    CLOCK_SYNC.lock().unwrap().update_offsets(packet);
    true
}
