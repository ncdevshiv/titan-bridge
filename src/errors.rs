use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(non_camel_case_types)]
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

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct SystemError {
    pub code: ErrorCode,
    pub module: &'static str,
    pub function: &'static str,
    pub message: String,
    pub context_dump: Option<String>,
}

impl SystemError {
    pub fn new(code: ErrorCode, module: &'static str, function: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            module,
            function,
            message: message.into(),
            context_dump: None,
        }
    }
    
    pub fn with_context(mut self, context: String) -> Self {
        self.context_dump = Some(context);
        self
    }
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] {}:{} -> {}", self.code, self.module, self.function, self.message)
    }
}

/// The Loud Failure Mandate macro
/// Forces a panic and explicit halt on critical registry errors, avoiding any silent fallbacks.
#[macro_export]
macro_rules! fail_loudly {
    ($err:expr) => {
        {
            crate::logger::log_critical_error(&$err);
            panic!("CRITICAL HALT: {} - {}", $err.code, $err.message);
        }
    };
}
