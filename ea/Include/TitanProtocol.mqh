//+------------------------------------------------------------------+
//|                                                 TitanProtocol.mqh|
//|                                       Titan Protocol Architecture|
//+------------------------------------------------------------------+
#property strict

// GLOBAL ERROR REGISTRY MAP
enum ENUM_SYSTEM_ERROR {
   ERR_001_ICEORYX_ROUDI_NOT_FOUND = 1,
   ERR_002_SHARED_MEMORY_ALLOCATION_LOCKED = 2,
   ERR_003_SPACETIMEDB_CONNECTION_FAILED = 3,
   
   ERR_100_NULL_POINTER_DEREFERENCE = 100,
   ERR_101_CRC_VALIDATION_FAILED = 101,
   ERR_102_SEQUENCE_DUPLICATION_DETECTED = 102,
   ERR_103_INVALID_PROTOBUF_PAYLOAD = 103,
   
   ERR_201_DATA_CIRCUIT_TRIPPED = 201,
   ERR_202_MAX_SLIPPAGE_EXCEEDED = 202,
   
   ERR_502_MT5_SYMBOL_NOT_FOUND = 502,
   ERR_503_MT5_DOM_SUBSCRIPTION_FAILED = 503
};

void FailLoudlyMQL(ENUM_SYSTEM_ERROR code, string module, string func, string reason) {
   string errMsg = StringFormat("CRITICAL HALT [%d]: %s::%s -> %s", code, module, func, reason);
   Print(errMsg);
   Alert(errMsg);
   ExpertRemove(); // Loudly rip the EA off the chart. No silent runs.
}

// Note: MQL5 doesn't support #pragma pack
// For C ABI alignment with Rust DLL, we rely on natural alignment
// All struct members are aligned to their natural boundaries

struct BinaryTick {
   uint symbol_id;
   long timestamp_ms;
   double bid;
   double ask;
   double last;
   ulong volume;
   ulong sequence_id;
   uint crc32;
};

struct ClockSyncPacket {
   long time_trade_server;
   ulong get_tick_count_64;
};

struct BinaryDOMLevel {
   double price;
   ulong volume;
};

struct BinaryDOM {
   uint symbol_id;
   long timestamp_ms;
   uint num_bids;
   uint num_asks;
   BinaryDOMLevel bids[20];
   BinaryDOMLevel asks[20];
   uint crc32;
};

struct MetadataPacket {
   uint symbol_id;
   double contract_size;
   double point_value;
   uint digits;
   long execution_mode;
};

// CRC32 Lookup Table for fast calculation matching fastcrc32 in Rust
uint crc32_table[256];
bool crc32_table_initialized = false;

void InitCRC32Table() {
   if (crc32_table_initialized) return;
   
   uint polynomial = 0xEDB88320;
   for (uint i = 0; i < 256; i++) {
      uint c = i;
      for (int j = 0; j < 8; j++) {
         if ((c & 1) != 0)
            c = polynomial ^ (c >> 1);
         else
            c >>= 1;
      }
      crc32_table[i] = c;
   }
   crc32_table_initialized = true;
}

// Helper function to get bytes from double (MQL5 doesn't have DoubleToLong)
void DoubleToBytes(double value, uchar &bytes[], int startIdx) {
   // Use StringToCharArray with binary representation
   string s = DoubleToString(value, 17);
   uchar temp[];
   int len = StringToCharArray(s, temp);
   for (int i = 0; i < 8 && i < len; i++) {
      bytes[startIdx + i] = temp[i];
   }
}

// Helper function to convert struct to byte array for CRC calculation
void TickToBytes(BinaryTick &tick, uchar &bytes[]) {
   ArrayResize(bytes, 56); // Size of BinaryTick minus crc32 field (4 bytes)
   int idx = 0;
   
   // Copy symbol_id (4 bytes)
   bytes[idx++] = (uchar)(tick.symbol_id & 0xFF);
   bytes[idx++] = (uchar)((tick.symbol_id >> 8) & 0xFF);
   bytes[idx++] = (uchar)((tick.symbol_id >> 16) & 0xFF);
   bytes[idx++] = (uchar)((tick.symbol_id >> 24) & 0xFF);
   
   // Copy timestamp_ms (8 bytes)
   for (int i = 0; i < 8; i++) {
      bytes[idx++] = (uchar)((tick.timestamp_ms >> (i * 8)) & 0xFF);
   }
   
   // Copy bid (8 bytes) - using simple byte extraction from double
   long bid_bits = (long)tick.bid;
   for (int i = 0; i < 8; i++) {
      bytes[idx++] = (uchar)((bid_bits >> (i * 8)) & 0xFF);
   }
   
   // Copy ask (8 bytes)
   long ask_bits = (long)tick.ask;
   for (int i = 0; i < 8; i++) {
      bytes[idx++] = (uchar)((ask_bits >> (i * 8)) & 0xFF);
   }
   
   // Copy last (8 bytes)
   long last_bits = (long)tick.last;
   for (int i = 0; i < 8; i++) {
      bytes[idx++] = (uchar)((last_bits >> (i * 8)) & 0xFF);
   }
   
   // Copy volume (8 bytes)
   for (int i = 0; i < 8; i++) {
      bytes[idx++] = (uchar)((tick.volume >> (i * 8)) & 0xFF);
   }
   
   // Copy sequence_id (8 bytes)
   for (int i = 0; i < 8; i++) {
      bytes[idx++] = (uchar)((tick.sequence_id >> (i * 8)) & 0xFF);
   }
}

// Helper function to convert DOM struct to byte array
void DOMToBytes(BinaryDOM &dom, uchar &bytes[]) {
   int numBidLevels = (int)dom.num_bids;
   int numAskLevels = (int)dom.num_asks;
   if (numBidLevels > 20) numBidLevels = 20;
   if (numAskLevels > 20) numAskLevels = 20;
   
   int size = 24 + (numBidLevels + numAskLevels) * 16; // Header + levels
   ArrayResize(bytes, size);
   int idx = 0;
   
   // Copy symbol_id (4 bytes)
   bytes[idx++] = (uchar)(dom.symbol_id & 0xFF);
   bytes[idx++] = (uchar)((dom.symbol_id >> 8) & 0xFF);
   bytes[idx++] = (uchar)((dom.symbol_id >> 16) & 0xFF);
   bytes[idx++] = (uchar)((dom.symbol_id >> 24) & 0xFF);
   
   // Copy timestamp_ms (8 bytes)
   for (int i = 0; i < 8; i++) {
      bytes[idx++] = (uchar)((dom.timestamp_ms >> (i * 8)) & 0xFF);
   }
   
   // Copy num_bids (4 bytes)
   bytes[idx++] = (uchar)(dom.num_bids & 0xFF);
   bytes[idx++] = (uchar)((dom.num_bids >> 8) & 0xFF);
   bytes[idx++] = (uchar)((dom.num_bids >> 16) & 0xFF);
   bytes[idx++] = (uchar)((dom.num_bids >> 24) & 0xFF);
   
   // Copy num_asks (4 bytes)
   bytes[idx++] = (uchar)(dom.num_asks & 0xFF);
   bytes[idx++] = (uchar)((dom.num_asks >> 8) & 0xFF);
   bytes[idx++] = (uchar)((dom.num_asks >> 16) & 0xFF);
   bytes[idx++] = (uchar)((dom.num_asks >> 24) & 0xFF);
   
   // Copy bids
   for (int i = 0; i < numBidLevels; i++) {
      long price_bits = (long)dom.bids[i].price;
      for (int j = 0; j < 8; j++) {
         bytes[idx++] = (uchar)((price_bits >> (j * 8)) & 0xFF);
      }
      for (int j = 0; j < 8; j++) {
         bytes[idx++] = (uchar)((dom.bids[i].volume >> (j * 8)) & 0xFF);
      }
   }
   
   // Copy asks
   for (int i = 0; i < numAskLevels; i++) {
      long price_bits = (long)dom.asks[i].price;
      for (int j = 0; j < 8; j++) {
         bytes[idx++] = (uchar)((price_bits >> (j * 8)) & 0xFF);
      }
      for (int j = 0; j < 8; j++) {
         bytes[idx++] = (uchar)((dom.asks[i].volume >> (j * 8)) & 0xFF);
      }
   }
}

uint CalculateBinaryTickCRC(BinaryTick &tick) {
   if (!crc32_table_initialized) InitCRC32Table();
   
   uint crc = 0xFFFFFFFF;
   uchar bytes[];
   TickToBytes(tick, bytes);
   int size = ArraySize(bytes);
   
   for (int i = 0; i < size; i++) {
      crc = crc32_table[(crc ^ bytes[i]) & 0xFF] ^ (crc >> 8);
   }
   return crc ^ 0xFFFFFFFF;
}

uint CalculateBinaryDOMCRC(BinaryDOM &dom) {
   if (!crc32_table_initialized) InitCRC32Table();
   
   uint crc = 0xFFFFFFFF;
   uchar bytes[];
   DOMToBytes(dom, bytes);
   int size = ArraySize(bytes);
   
   for (int i = 0; i < size; i++) {
      crc = crc32_table[(crc ^ bytes[i]) & 0xFF] ^ (crc >> 8);
   }
   return crc ^ 0xFFFFFFFF;
}
