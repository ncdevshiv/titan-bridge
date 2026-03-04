// This file interfaces with the C++ Eclipse Iceoryx POSH API via `cxx` crate.
use cxx::UniquePtr;
use crate::CONFIG;

#[cxx::bridge(namespace = "titan::iceoryx")]
pub mod ffi {
    unsafe extern "C++" {
        include!("titan_iceoryx.hpp");

        type IceoryxNode;

        fn create_iceoryx_node(app_name: &str) -> UniquePtr<IceoryxNode>;
        unsafe fn publish_tick(self: &IceoryxNode, payload: *const u8, size: usize) -> bool;
        unsafe fn publish_book(self: &IceoryxNode, payload: *const u8, size: usize) -> bool;
        unsafe fn publish_metadata(self: &IceoryxNode, payload: *const u8, size: usize) -> bool;
        fn send_command(self: &IceoryxNode, cmd: &str) -> bool;
        fn pool_usage_percent(self: &IceoryxNode) -> f64;
    }
}

pub struct SharedMemoryContext {
    node: UniquePtr<ffi::IceoryxNode>,
}

unsafe impl Send for SharedMemoryContext {}
unsafe impl Sync for SharedMemoryContext {}

impl SharedMemoryContext {
    pub fn new(app_name: &str) -> Self {
        crate::logger::log_info("shared_memory", "init", "Initializing Eclipse Iceoryx POSH node.");
        Self {
            node: ffi::create_iceoryx_node(app_name),
        }
    }

    pub fn publish_tick_bytes(&self, bytes: &[u8]) -> bool {
        self.check_backpressure();
        unsafe { self.node.publish_tick(bytes.as_ptr(), bytes.len()) }
    }

    pub fn publish_book_bytes(&self, bytes: &[u8]) -> bool {
        self.check_backpressure();
        unsafe { self.node.publish_book(bytes.as_ptr(), bytes.len()) }
    }

    pub fn publish_metadata_bytes(&self, bytes: &[u8]) -> bool {
        unsafe { self.node.publish_metadata(bytes.as_ptr(), bytes.len()) }
    }

    pub fn check_backpressure(&self) {
        let usage = self.node.pool_usage_percent();
        if usage > CONFIG.iceoryx_threshold_percent {
            crate::logger::log_warn("shared_memory", "check_backpressure", &format!("BACKPRESSURE_WARNING: Iceoryx pool capacity > {}%", CONFIG.iceoryx_threshold_percent));
            // Send backpressure warning to MQL5 command buffer subscriber
            self.send_command("BACKPRESSURE_WARNING");
        }
    }

    pub fn send_command(&self, cmd: &str) -> bool {
        self.node.send_command(cmd)
    }
}
