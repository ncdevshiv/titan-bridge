use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;

use crate::errors::SystemError;

pub fn log_info(module: &str, function: &str, message: &str) {
    log_event("INFO", module, function, message, None, None);
}

pub fn log_warn(module: &str, function: &str, message: &str) {
    log_event("WARN", module, function, message, None, None);
}

pub fn log_error(module: &str, function: &str, message: &str, error_details: Option<&str>) {
    log_event("ERROR", module, function, message, error_details, None);
}

pub fn log_critical_error(err: &SystemError) {
    let context = err.context_dump.as_deref();
    log_event("CRITICAL", err.module, err.function, &format!("{} [{}]", err.message, err.code), context, Some(format!("{:?}", err.code)));
}

fn log_event(level: &str, module: &str, function: &str, message: &str, context: Option<&str>, error_code: Option<String>) {
    let mut log_obj = json!({
        "timestamp": Utc::now().to_rfc3339(),
        "level": level,
        "module": module,
        "function": function,
        "message": message,
    });

    if let Some(ctx) = context {
        log_obj["context"] = json!(ctx);
    }
    
    if let Some(code) = error_code {
        log_obj["error_code"] = json!(code);
    }

    let log_str = log_obj.to_string();
    
    // Output directly to stdout for frontend visibility
    println!("{}", log_str);

    // Also write to a file
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("titan_bridge.log")
    {
        let _ = writeln!(file, "{}", log_str);
    }
}
