use std::time::{Instant, Duration};
use crate::errors::{ErrorCode, SystemError};

pub struct MaxSlippageGuard {
    max_slippage_points: f64,
    last_known_bid: f64,
    last_known_ask: f64,
}

impl MaxSlippageGuard {
    pub fn new(max_slippage_points: f64) -> Self {
        Self {
            max_slippage_points,
            last_known_bid: 0.0,
            last_known_ask: 0.0,
        }
    }

    pub fn update_prices(&mut self, bid: f64, ask: f64) {
        self.last_known_bid = bid;
        self.last_known_ask = ask;
    }

    pub fn validate_execution(&self, expected_price: f64, is_buy: bool) -> Result<(), SystemError> {
        let current_price = if is_buy { self.last_known_ask } else { self.last_known_bid };
        let slippage = (current_price - expected_price).abs();

        if slippage > self.max_slippage_points {
            Err(SystemError::new(ErrorCode::ERR_202_MAX_SLIPPAGE_EXCEEDED, "circuit_breaker", "validate_execution", format!("Order rejected: slippage {} > max {}", slippage, self.max_slippage_points)))
        } else {
            Ok(())
        }
    }
}

pub struct DataCircuitBreaker {
    max_tps: u32,
    tick_count: u32,
    window_start: Instant,
    is_tripped: bool,
    trip_duration: Duration,
    tripped_at: Option<Instant>,
}

impl DataCircuitBreaker {
    pub fn new(max_tps: u32, trip_duration_ms: u64) -> Self {
        Self {
            max_tps,
            tick_count: 0,
            window_start: Instant::now(),
            is_tripped: false,
            trip_duration: Duration::from_millis(trip_duration_ms),
            tripped_at: None,
        }
    }

    pub fn allow_tick(&mut self) -> bool {
        let now = Instant::now();

        if self.is_tripped {
            if let Some(trip_time) = self.tripped_at {
                if now.duration_since(trip_time) >= self.trip_duration {
                    self.reset(now);
                } else {
                    return false;
                }
            }
        }

        if now.duration_since(self.window_start) >= Duration::from_secs(1) {
            self.window_start = now;
            self.tick_count = 0;
        }

        self.tick_count += 1;

        if self.tick_count > self.max_tps {
            self.trip(now);
            return false;
        }

        true
    }

    fn trip(&mut self, time: Instant) {
        self.is_tripped = true;
        self.tripped_at = Some(time);
        
        let err = SystemError::new(ErrorCode::ERR_201_DATA_CIRCUIT_TRIPPED, "circuit_breaker", "data_circuit_breaker", "Data burst detected: Tripped ingest circuit breaker");
        crate::logger::log_critical_error(&err);
    }

    fn reset(&mut self, time: Instant) {
        self.is_tripped = false;
        self.tripped_at = None;
        self.window_start = time;
        self.tick_count = 0;
        crate::logger::log_info("circuit_breaker", "data_circuit_breaker", "Circuit breaker reset: Resuming ingestion");
    }
}
