//+------------------------------------------------------------------+
//|                                               IceoryxMemPool.mqh |
//|                                       Titan Protocol Architecture|
//+------------------------------------------------------------------+
#property strict
#include "TitanProtocol.mqh"

#import "titan_bridge.dll"
bool bridge_init();
bool bridge_push_tick(const BinaryTick& tick);
bool bridge_push_book(const BinaryDOM& dom);
bool bridge_sync_clock(const ClockSyncPacket& packet);
bool bridge_push_metadata(const MetadataPacket& packet);
#import

class CIceoryxMemPool {
private:
   bool is_initialized;
   bool backpressure_warning_active;

public:
   CIceoryxMemPool() : is_initialized(false), backpressure_warning_active(false) {}
   
   bool Initialize() {
      Print("Initializing Titan Bridge DLL...");
      is_initialized = bridge_init();
      if (!is_initialized) {
         Print("Failed to initialize Titan Bridge DLL.");
      }
      return is_initialized;
   }
   
   bool PublishTick(BinaryTick& tick) {
      if (!is_initialized) return false;
      tick.crc32 = CalculateBinaryTickCRC(tick);
      return bridge_push_tick(tick);
   }

   bool PublishDOM(BinaryDOM& dom) {
      if (!is_initialized) return false;
      dom.crc32 = CalculateBinaryDOMCRC(dom);
      return bridge_push_book(dom);
   }
   
   bool PublishClockSync(const ClockSyncPacket& packet) {
      if (!is_initialized) return false;
      return bridge_sync_clock(packet);
   }

   bool PublishMetadata(const MetadataPacket& packet) {
      if (!is_initialized) return false;
      return bridge_push_metadata(packet);
   }
   
   void CheckBackpressureCmd(string cmd) {
      if (cmd == "BACKPRESSURE_WARNING") {
         Print("Received BACKPRESSURE_WARNING from rust. Throttling DOM.");
         backpressure_warning_active = true;
      } else if (cmd == "BACKPRESSURE_RESOLVED") {
         backpressure_warning_active = false;
      }
   }
   
   bool IsThrottled() const {
      return backpressure_warning_active;
   }
};

CIceoryxMemPool MemPool;
