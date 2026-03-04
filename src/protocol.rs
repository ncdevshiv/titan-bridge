use std::collections::HashMap;
use crc32fast::Hasher;
use crate::errors::{ErrorCode, SystemError};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BinaryTick {
    pub symbol_id: u32,
    pub timestamp_ms: i64,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: u64,
    pub sequence_id: u64,
    pub crc32: u32,
}

pub struct ProtocolNormalizer {
    last_processed_sequence: HashMap<u32, u64>,
}

impl ProtocolNormalizer {
    pub fn new() -> Self {
        Self {
            last_processed_sequence: HashMap::new(),
        }
    }

    /// Verifies the payload integrity using crc32fast
    pub fn verify_integrity(tick: &BinaryTick) -> bool {
        let mut hasher = Hasher::new();
        hasher.update(&tick.symbol_id.to_le_bytes());
        hasher.update(&tick.timestamp_ms.to_le_bytes());
        hasher.update(&tick.bid.to_le_bytes());
        hasher.update(&tick.ask.to_le_bytes());
        hasher.update(&tick.last.to_le_bytes());
        hasher.update(&tick.volume.to_le_bytes());
        hasher.update(&tick.sequence_id.to_le_bytes());
        
        let calculated_crc = hasher.finalize();
        calculated_crc == tick.crc32
    }

    /// Ghost Tick Filter (Deduplication)
    /// Discards duplicate or out-of-sequence ticks
    pub fn process_tick(&mut self, tick: &BinaryTick) -> Result<(), SystemError> {
        if !Self::verify_integrity(tick) {
            return Err(SystemError::new(ErrorCode::ERR_101_CRC_VALIDATION_FAILED, "protocol", "process_tick", "CRC validation failed (corrupt DMA transfer)"));
        }

        let last_seq = self.last_processed_sequence.entry(tick.symbol_id).or_insert(0);
        
        if tick.sequence_id <= *last_seq && *last_seq != 0 {
            let msg = format!("Ghost tick rejected - symbol_id: {}, seq: {}", tick.symbol_id, tick.sequence_id);
            crate::logger::log_warn("protocol", "process_tick", &msg);
            
            // We treat duplicates as standard warnings to drop safely, 
            // but out-of-order bounds might trigger explicit fail states in tests
            return Ok(());
        }

        if tick.sequence_id > *last_seq + 1 && *last_seq != 0 {
            // Gap detected! We might need historic refill sync
            crate::logger::log_warn("protocol", "process_tick", &format!("Sequence gap detected - symbol_id: {}, expected: {}, got: {}", tick.symbol_id, *last_seq + 1, tick.sequence_id));
        }

        *last_seq = tick.sequence_id;
        Ok(())
    }
}
