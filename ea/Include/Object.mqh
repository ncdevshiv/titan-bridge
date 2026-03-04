//+------------------------------------------------------------------+
//|                                                     Object.mqh   |
//|                        MetaTrader 5 Standard Library             |
//+------------------------------------------------------------------+
#property strict

// Standard base class for MQL5 library objects
class CObject
{
public:
                     CObject(void)                     { }
                    ~CObject(void)                     { }
   
   // Comparison method
   virtual int       Compare(const CObject *node, const int mode=0) const { return(0); }
   
   // Search method
   virtual bool      Search(const CObject *node) const                   { return(false); }
   
   // Type identification
   virtual int       Type(void) const                                    { return(0); }
};
