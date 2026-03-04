//+------------------------------------------------------------------+
//|                                              ParallelEngine.mqh  |
//|                                       Titan Protocol Architecture|
//+------------------------------------------------------------------+
#property strict

#include <Trade\Trade.mqh>

struct BulkOrder {
   ulong ticket;
   string symbol;
   int type;      // ORDER_TYPE_BUY, ORDER_TYPE_SELL, etc.
   double volume;
   double price;
   double sl;
   double tp;
   ulong link_id; // For OCA/OCO linked orders
};

class CParallelEngine {
private:
   CTrade trade;
   BulkOrder order_queue[];
   int queue_size;

public:
   CParallelEngine() {
      queue_size = 0;
   }
   
   void EnqueueOrder(const BulkOrder& order) {
      ArrayResize(order_queue, queue_size + 1);
      order_queue[queue_size] = order;
      queue_size++;
   }
   
   void ExecuteBulk() {
      if (queue_size == 0) return;
      
      // Attempting bulk execution
      for (int i = 0; i < queue_size; i++) {
         double vol = order_queue[i].volume;
         string sym = order_queue[i].symbol;
         double prc = order_queue[i].price;
         double sl  = order_queue[i].sl;
         double tp  = order_queue[i].tp;
         
         if (order_queue[i].type == ORDER_TYPE_BUY) {
            if (!trade.Buy(vol, sym, prc, sl, tp)) {
               Print("Buy execution failed: ", trade.ResultRetcodeDescription());
            } else {
               HandleLinkedOrders(order_queue[i].link_id);
            }
         } else if (order_queue[i].type == ORDER_TYPE_SELL) {
            if (!trade.Sell(vol, sym, prc, sl, tp)) {
               Print("Sell execution failed: ", trade.ResultRetcodeDescription());
            } else {
               HandleLinkedOrders(order_queue[i].link_id);
            }
         }
         // Add explicit execution of pending stops/limits that trigger OCO links
         else if (order_queue[i].type == ORDER_TYPE_BUY_LIMIT || order_queue[i].type == ORDER_TYPE_BUY_STOP) {
            if (!trade.BuyLimit(vol, prc, sym, sl, tp)) {
               Print("Pending order failed: ", trade.ResultRetcodeDescription());
            }
         }
      }
      
      // Clear queue
      ArrayFree(order_queue);
      queue_size = 0;
   }
   
private:
   void HandleLinkedOrders(ulong link_id) {
      if (link_id == 0) return; // Unlinked order
      
      Print("Order matched link_id: ", link_id, " - executing local OCO cancellation.");
      // Real implementation replacing stub
      for (int i = OrdersTotal() - 1; i >= 0; i--) {
          ulong ticket = OrderGetTicket(i);
          if (ticket > 0) {
              // Extract the link ID (e.g., from comment or magic number based on trading strategy mapping)
              // For purely local implementation, relying on MQL5 Magic Number or comment prefix
              long magic = OrderGetInteger(ORDER_MAGIC);
              if (magic == link_id) {
                  Print("OCO Triggered - Canceling sibling pending order: ", ticket);
                  if (!trade.OrderDelete(ticket)) {
                      Print("Failed to cancel OCO linked order: ", trade.ResultRetcodeDescription());
                  }
              }
          }
      }
   }
};
