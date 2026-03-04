#pragma once
#include <string>
#include <memory>
#include <atomic>
#include "rust/cxx.h"
// Actual iceoryx includes
#include "iceoryx_posh/popo/publisher.hpp"
#include "iceoryx_posh/runtime/posh_runtime.hpp"

namespace titan {
namespace iceoryx {

using namespace iox::popo;

class IceoryxNode {
private:
    std::unique_ptr<Publisher<uint8_t>> tick_publisher;
    std::unique_ptr<Publisher<uint8_t>> book_publisher;
    std::unique_ptr<Publisher<uint8_t>> metadata_publisher;
    std::unique_ptr<Publisher<uint8_t>> command_publisher;

    // Pool usage tracking metrics (mutable for const-correctness)
    mutable std::atomic<uint64_t> total_publish_attempts{0};
    mutable std::atomic<uint64_t> failed_loan_attempts{0};

public:
    IceoryxNode(rust::Str app_name);
    bool publish_tick(const uint8_t* payload, size_t size) const;
    bool publish_book(const uint8_t* payload, size_t size) const;
    bool publish_metadata(const uint8_t* payload, size_t size) const;
    bool send_command(rust::Str cmd) const;
    double pool_usage_percent() const;
};

std::unique_ptr<IceoryxNode> create_iceoryx_node(rust::Str app_name);

} // namespace iceoryx
} // namespace titan
