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


// Note: Strict packing to ensure C ABI alignment with Rust DLL
#pragma pack(push, 1)

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

#pragma pack(pop)

// CRC32 Lookup Table for fast calculation matching fastcrc32 in Rust
uint crc32_table[256];

void InitCRC32Table() {
   uint polynomial = 0xEDB88320;
   for (uint i = 0; i < 256; i++) {
      uint c = i;
      for (int j = 0; j < 8; j++) {
         if (c & 1)
            c = polynomial ^ (c >> 1);
         else
            c >>= 1;
      }
      crc32_table[i] = c;
   }
}

uint CalculateBinaryTickCRC(const BinaryTick& tick) {
   uint crc = 0xFFFFFFFF;
   const int size = sizeof(BinaryTick) - sizeof(uint);
   uchar bytes[];
   StructToCharArray(tick, bytes);
   for (int i = 0; i < size; i++) {
      crc = crc32_table[(crc ^ bytes[i]) & 0xFF] ^ (crc >> 8);
   }
   return crc ^ 0xFFFFFFFF;
}

uint CalculateBinaryDOMCRC(const BinaryDOM& dom) {
   uint crc = 0xFFFFFFFF;
   const int size = sizeof(BinaryDOM) - sizeof(uint);
   uchar bytes[];
   StructToCharArray(dom, bytes);
   for (int i = 0; i < size; i++) {
      crc = crc32_table[(crc ^ bytes[i]) & 0xFF] ^ (crc >> 8);
   }
   return crc ^ 0xFFFFFFFF;
}
