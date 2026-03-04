//+------------------------------------------------------------------+
//|                                     HighPerformanceBridgeEA.mq5  |
//|                                       Titan Protocol Architecture|
//+------------------------------------------------------------------+
#property copyright "Titan Protocol"
#property link      ""
#property version   "1.01"

#include <TitanBridge\IceoryxMemPool.mqh>
#include <TitanBridge\ParallelEngine.mqh>
#include <TitanBridge\Diagnostics.mqh>

input int DOM_Depth = 20;

// Symbol identifier cache to avoid allocating strings inside the hot path
uint symbol_id_cache; 
CParallelEngine BulkEngine;

// Simple deterministic hash for Symbol string to u32 mapping
uint HashString(string str) {
   uint hash = 5381;
   for (int i = 0; i < StringLen(str); i++) {
      hash = ((hash << 5) + hash) + StringGetCharacter(str, i);
   }
   return hash;
}

int OnInit() {
   Print("Starting High Performance Bridge EA...");
   Print("Session UUID: ", SessionDiagnostics.GetSessionUUID());
   
   InitCRC32Table();
   
   if (!MemPool.Initialize()) {
      FailLoudlyMQL(ERR_001_ICEORYX_ROUDI_NOT_FOUND, "HighPerformanceBridgeEA", "OnInit", "Failed to attach to iceoryx memory pool.");
      return INIT_FAILED;
   }
   
   // We cache a simple hash of the symbol name as the symbol_id 
   symbol_id_cache = HashString(Symbol());
   
   if (symbol_id_cache == 0 || Symbol() == "") {
      FailLoudlyMQL(ERR_502_MT5_SYMBOL_NOT_FOUND, "HighPerformanceBridgeEA", "OnInit", "Unable to resolve active chart symbol");
      return INIT_FAILED;
   }
   
   EventSetMillisecondTimer(1); // 1-millisecond resolution
   if (!MarketBookAdd(Symbol())) {
      FailLoudlyMQL(ERR_503_MT5_DOM_SUBSCRIPTION_FAILED, "HighPerformanceBridgeEA", "OnInit", "MarketBookAdd failed. Check broker permissions.");
      return INIT_FAILED;
   }
   
   return INIT_SUCCEEDED;
}

void OnDeinit(const int reason) {
   EventKillTimer();
   MarketBookRelease(Symbol());
   Print("Bridge EA Deinitialized.");
}

void OnTimer() {
   // 1ms precision timer
   static ulong last_heartbeat = 0;
   ulong now = GetTickCount64();
   
   if (now - last_heartbeat > 5000) { // Every 5 seconds
      last_heartbeat = now;
      SessionDiagnostics.BroadcastHeartbeat(symbol_id_cache);
      SessionDiagnostics.SendClockSyncPacket();
   }
   
   BulkEngine.ExecuteBulk();
}

void OnTick() {
   MqlTick tick;
   if (!SymbolInfoTick(Symbol(), tick)) return;
   
   static ulong sequence = 0;
   
   BinaryTick bt;
   bt.symbol_id = symbol_id_cache;
   bt.timestamp_ms = tick.time_msc;
   bt.bid = tick.bid;
   bt.ask = tick.ask;
   bt.last = tick.last;
   bt.volume = tick.volume;
   bt.sequence_id = ++sequence;
   
   if (!MemPool.PublishTick(bt)) {
      Print("Failed to publish tick #", sequence);
   }
}

void OnBookEvent(const string& symbol) {
   if (MemPool.IsThrottled()) return; // Drops DOM frames if Backpressure warning active
   
   MqlBookInfo book[];
   if (MarketBookGet(symbol, book)) {
      int total_levels = ArraySize(book);
      int max_levels = MathMin(total_levels, 20); // Hard maximum 20 levels in struct
      
      BinaryDOM dom;
      dom.symbol_id = symbol_id_cache;
      dom.timestamp_ms = TimeCurrent() * 1000;
      dom.num_bids = 0;
      dom.num_asks = 0;
      
      // Real DOM Serialization to remove stub
      for (int i = 0; i < total_levels; i++) {
          if (book[i].type == BOOK_TYPE_BUY && dom.num_bids < 20) {
              dom.bids[dom.num_bids].price = book[i].price;
              dom.bids[dom.num_bids].volume = (ulong)book[i].volume;
              dom.num_bids++;
          } else if (book[i].type == BOOK_TYPE_SELL && dom.num_asks < 20) {
              dom.asks[dom.num_asks].price = book[i].price;
              dom.asks[dom.num_asks].volume = (ulong)book[i].volume;
              dom.num_asks++;
          }
      }
      
      if (!MemPool.PublishDOM(dom)) {
          // Silent failure on DOM drops to prevent log spam in micro-bursts
      }
   }
}
