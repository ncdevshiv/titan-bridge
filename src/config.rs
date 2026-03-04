use serde::Deserialize;
use std::fs;
use crate::errors::{ErrorCode, SystemError};

#[derive(Deserialize, Debug, Clone)]
pub struct SystemConfig {
    pub max_tps: u32,
    pub trip_duration_ms: u64,
    pub max_slippage_points: f64,
    pub iceoryx_threshold_percent: f64,
    pub app_name: String,
}

impl SystemConfig {
    pub fn load() -> Result<Self, SystemError> {
        let json_data = fs::read_to_string("config/system.json")
            .map_err(|e| SystemError::new(
                ErrorCode::ERR_004_CONFIG_FILE_ERROR,
                "config",
                "load",
                format!("CRITICAL: Missing config/system.json. {}", e)
            ))?;
            
        serde_json::from_str(&json_data)
            .map_err(|e| SystemError::new(
                ErrorCode::ERR_004_CONFIG_FILE_ERROR,
                "config",
                "load",
                format!("CRITICAL: Malformed config/system.json. {}", e)
            ))
    }
}
