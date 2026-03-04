use std::time::{SystemTime, UNIX_EPOCH};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ClockSyncPacket {
    pub time_trade_server: i64,
    pub get_tick_count_64: u64,
}

pub struct TripleClock {
    pub broker_server_offset_ms: f64,
    pub ptp_offset: f64,
    alpha: f64,
}

impl TripleClock {
    pub fn new() -> Self {
        Self {
            broker_server_offset_ms: 0.0,
            ptp_offset: 0.0,
            alpha: 0.1, // Exponential Moving Average smoothing factor
        }
    }

    /// Calculates precise latency offset taking the Triple Clock problem into account
    /// to ensure chronologically perfect historic replay in DuckDB.
    pub fn update_offsets(&mut self, packet: &ClockSyncPacket) {
        let local_time_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
            
        let new_broker_offset = (local_time_ms - packet.time_trade_server) as f64;
        let new_ptp_offset = (local_time_ms - packet.get_tick_count_64 as i64) as f64;

        if self.broker_server_offset_ms == 0.0 {
            // Initial sync
            self.broker_server_offset_ms = new_broker_offset;
            self.ptp_offset = new_ptp_offset;
        } else {
            // Apply EMA
            self.broker_server_offset_ms = (self.alpha * new_broker_offset) + ((1.0 - self.alpha) * self.broker_server_offset_ms);
            self.ptp_offset = (self.alpha * new_ptp_offset) + ((1.0 - self.alpha) * self.ptp_offset);
        }
        
        crate::logger::log_info(
            "clock_sync", 
            "update_offsets", 
            &format!("PTP Offset (EMA): {:.2} ms, Broker Offset (EMA): {:.2} ms", self.ptp_offset, self.broker_server_offset_ms)
        );
    }
}
