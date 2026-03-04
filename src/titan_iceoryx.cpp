#include "titan_iceoryx.hpp"
#include "iceoryx_hoofs/cxx/string.hpp"

namespace titan {
namespace iceoryx {

IceoryxNode::IceoryxNode(rust::Str app_name) {
    iox::runtime::PoshRuntime::initRuntime(iox::RuntimeName_t(iox::cxx::TruncateToCapacity, std::string(app_name).c_str()));
    
    // Abstracting Topic Maps for Consumer Plug-and-Play
    // Topics: "Titan", "MarketData", <Event>
    tick_publisher = std::make_unique<Publisher<uint8_t>>(
        iox::capro::ServiceDescription("Titan", "MarketData", "TickStream"));
        
    book_publisher = std::make_unique<Publisher<uint8_t>>(
        iox::capro::ServiceDescription("Titan", "MarketData", "DepthOfMarket"));

    metadata_publisher = std::make_unique<Publisher<uint8_t>>(
        iox::capro::ServiceDescription("Titan", "MarketData", "Metadata"));

    command_publisher = std::make_unique<Publisher<uint8_t>>(
        iox::capro::ServiceDescription("Titan", "Commands", "Control"));
        
    tick_publisher->offer();
    book_publisher->offer();
    metadata_publisher->offer();
    command_publisher->offer();
}

bool IceoryxNode::publish_tick(const uint8_t* payload, size_t size) const {
    total_publish_attempts.fetch_add(1, std::memory_order_relaxed);
    auto sample = tick_publisher->loan();
    if (!sample.has_error()) {
        std::memcpy(sample.value().get(), payload, size);
        sample.value().publish();
        return true;
    }
    failed_loan_attempts.fetch_add(1, std::memory_order_relaxed);
    return false;
}

bool IceoryxNode::publish_book(const uint8_t* payload, size_t size) const {
    total_publish_attempts.fetch_add(1, std::memory_order_relaxed);
    auto sample = book_publisher->loan();
    if (!sample.has_error()) {
        std::memcpy(sample.value().get(), payload, size);
        sample.value().publish();
        return true;
    }
    failed_loan_attempts.fetch_add(1, std::memory_order_relaxed);
    return false;
}

bool IceoryxNode::publish_metadata(const uint8_t* payload, size_t size) const {
    total_publish_attempts.fetch_add(1, std::memory_order_relaxed);
    auto sample = metadata_publisher->loan();
    if (!sample.has_error()) {
        std::memcpy(sample.value().get(), payload, size);
        sample.value().publish();
        return true;
    }
    failed_loan_attempts.fetch_add(1, std::memory_order_relaxed);
    return false;
}

bool IceoryxNode::send_command(rust::Str cmd) const {
    total_publish_attempts.fetch_add(1, std::memory_order_relaxed);

    // Serialize command string to bytes
    std::string cmd_str(cmd.data(), cmd.size());
    size_t cmd_size = cmd_str.size();

    auto sample = command_publisher->loan();
    if (!sample.has_error()) {
        std::memcpy(sample.value().get(), cmd_str.data(), cmd_size);
        sample.value().publish();
        return true;
    }

    failed_loan_attempts.fetch_add(1, std::memory_order_relaxed);
    return false;
}

double IceoryxNode::pool_usage_percent() const {
    // Calculate memory pressure based on failed loan attempts vs total attempts
    // This gives us a metric of how often we're hitting backpressure
    uint64_t total = total_publish_attempts.load(std::memory_order_relaxed);
    uint64_t failed = failed_loan_attempts.load(std::memory_order_relaxed);

    if (total == 0) {
        return 0.0;  // No attempts yet, no pressure
    }

    // Calculate percentage: (failed / total) * 100
    // Cap at 100.0 for safety
    double percent = (static_cast<double>(failed) / static_cast<double>(total)) * 100.0;
    return percent > 100.0 ? 100.0 : percent;
}

std::unique_ptr<IceoryxNode> create_iceoryx_node(rust::Str app_name) {
    return std::make_unique<IceoryxNode>(app_name);
}

} // namespace iceoryx
} // namespace titan
