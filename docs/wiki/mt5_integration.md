# MetaTrader 5 Integration Guide

Complete guide for integrating the Titan Bridge DLL with MetaTrader 5 for ultra-low latency market data streaming.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation Steps](#installation-steps)
3. [Configuration](#configuration)
4. [Loading the EA](#loading-the-ea)
5. [Testing Procedures](#testing-procedures)
6. [Troubleshooting](#troubleshooting)
7. [Verification Checklist](#verification-checklist)

---

## Prerequisites

Before integrating Titan Bridge with MT5, ensure the following components are ready:

### System Requirements

| Component | Minimum Requirement |
|-----------|---------------------|
| Operating System | Windows 10/11 (64-bit) |
| MetaTrader 5 | Build 4000+ |
| RAM | 8 GB (16 GB recommended) |
| Free Disk Space | 2 GB for logs and temp files |
| CPU | Multi-core processor recommended |

### Required Components

1. **MetaTrader 5 Installed and Running**
   - Terminal must be from a broker that supports Level 2 DOM data
   - Verify DOM access permissions with your broker

2. **Titan Bridge DLL Built**
   ```
   target/release/titan_bridge.dll
   ```
   - Build with `cargo build --release`
   - Must be compiled with dynamic CRT linking (see `.cargo/config.toml`)

3. **Iceoryx RouDi Daemon Running**
   - The RouDi daemon must be started before MT5 loads the EA
   - Built from `third_party/iceoryx` during bootstrap
   - Executable location: `third_party/iceoryx/build/iox-roudi`

### Verify Prerequisites

```powershell
# Check Rust installation
rustc --version

# Check CMake installation
cmake --version

# Verify DLL exists
Test-Path "target/release/titan_bridge.dll"

# Check RouDi is running (in PowerShell)
Get-Process | Where-Object { $_.ProcessName -like "*roudi*" }
```

---

## Installation Steps

### Step 1: Locate Your MT5 Data Folder

The MT5 data folder path varies by installation type:

| Installation Type | Default Path |
|-------------------|--------------|
| Standard | `C:\Users\<Username>\AppData\Roaming\MetaQuotes\Terminal\<Hash>\` |
| Portable | `<MT5_Install_Dir>\` |

To find your MT5 data folder:
1. Open MetaTrader 5
2. Click **File** → **Open Data Folder**
3. Note the path - this is your `<MT5_DATA>` path

### Step 2: Create Folder Structure

Create the following folder structure inside your MT5 data folder:

```
<MT5_DATA_FOLDER>/
├── MQL5/
│   ├── Experts/
│   │   └── TitanBridge/          # Create this folder
│   ├── Include/
│   │   └── TitanBridge/          # Create this folder
│   └── Libraries/
│       └── TitanBridge/          # Create this folder
```

PowerShell command to create structure:
```powershell
$mt5Path = "C:\Users\$env:USERNAME\AppData\Roaming\MetaQuotes\Terminal\<YOUR_HASH>"
New-Item -ItemType Directory -Force -Path "$mt5Path\MQL5\Experts\TitanBridge"
New-Item -ItemType Directory -Force -Path "$mt5Path\MQL5\Include\TitanBridge"
New-Item -ItemType Directory -Force -Path "$mt5Path\MQL5\Libraries\TitanBridge"
```

### Step 3: Copy the DLL

Copy the Titan Bridge DLL to the Libraries folder:

**Source:** `target/release/titan_bridge.dll`
**Destination:** `<MT5_DATA_FOLDER>/MQL5/Libraries/TitanBridge/titan_bridge.dll`

```powershell
Copy-Item "target/release/titan_bridge.dll" `
  "$mt5Path\MQL5\Libraries\TitanBridge\titan_bridge.dll"
```

### Step 4: Copy EA Files

Copy the Expert Advisor and include files:

**EA File:**
- Source: `ea/HighPerformanceBridgeEA.mq5`
- Destination: `<MT5_DATA_FOLDER>/MQL5/Experts/TitanBridge/HighPerformanceBridgeEA.mq5`

**Include Files:**
- Source: `ea/Include/TitanProtocol.mqh`
- Destination: `<MT5_DATA_FOLDER>/MQL5/Include/TitanBridge/TitanProtocol.mqh`

- Source: `ea/Include/IceoryxMemPool.mqh`
- Destination: `<MT5_DATA_FOLDER>/MQL5/Include/TitanBridge/IceoryxMemPool.mqh`

- Source: `ea/Include/Diagnostics.mqh`
- Destination: `<MT5_DATA_FOLDER>/MQL5/Include/TitanBridge/Diagnostics.mqh`

- Source: `ea/Include/ParallelEngine.mqh`
- Destination: `<MT5_DATA_FOLDER>/MQL5/Include/TitanBridge/ParallelEngine.mqh`

PowerShell one-liner:
```powershell
Copy-Item "ea/HighPerformanceBridgeEA.mq5" "$mt5Path\MQL5\Experts\TitanBridge\"
Copy-Item "ea/Include/*.mqh" "$mt5Path\MQL5\Include\TitanBridge\"
```

### Step 5: Create Configuration File

Create the system configuration file in the Titan Bridge project directory:

**File:** `config/system.json`

```json
{
  "max_tps": 10000,
  "trip_duration_ms": 5000,
  "max_slippage_points": 5.0,
  "iceoryx_threshold_percent": 80.0,
  "app_name": "TitanBridge"
}
```

Configuration parameters:

| Parameter | Description | Default |
|-----------|-------------|---------|
| `max_tps` | Maximum ticks per second before circuit breaker trips | 10000 |
| `trip_duration_ms` | Duration to pause intake when circuit breaker trips (ms) | 5000 |
| `max_slippage_points` | Maximum acceptable slippage in points | 5.0 |
| `iceoryx_threshold_percent` | Memory threshold warning percentage | 80.0 |
| `app_name` | Application name for Iceoryx service | "TitanBridge" |

### Step 6: Update Include Paths in EA

Edit `HighPerformanceBridgeEA.mq5` to update the include paths:

```mql5
// Change from:
#include "Include\IceoryxMemPool.mqh"
#include "Include\ParallelEngine.mqh"
#include "Include\Diagnostics.mqh"

// To:
#include "..\..\Include\TitanBridge\IceoryxMemPool.mqh"
#include "..\..\Include\TitanBridge\ParallelEngine.mqh"
#include "..\..\Include\TitanBridge\Diagnostics.mqh"
```

Also update the DLL import path in `IceoryxMemPool.mqh`:

```mql5
// Change from:
#import "titan_bridge.dll"

// To:
#import "..\..\Libraries\TitanBridge\titan_bridge.dll"
```

### Final Folder Structure

```
<MT5_DATA_FOLDER>/
├── MQL5/
│   ├── Experts/
│   │   └── TitanBridge/
│   │       └── HighPerformanceBridgeEA.mq5
│   ├── Include/
│   │   └── TitanBridge/
│   │       ├── TitanProtocol.mqh
│   │       ├── IceoryxMemPool.mqh
│   │       ├── Diagnostics.mqh
│   │       └── ParallelEngine.mqh
│   └── Libraries/
│       └── TitanBridge/
│           └── titan_bridge.dll
```

---

## Configuration

### Symbol Setup

1. **Open the Market Watch window** (Ctrl+M)
2. **Right-click** and select **Symbols**
3. **Add the desired trading symbols** to Market Watch
4. **Verify DOM availability:**
   - Right-click on a symbol in Market Watch
   - Select **Depth of Market**
   - Ensure Level 2 data is displayed

### Configuring EA Inputs

The EA has one configurable input parameter:

```mql5
input int DOM_Depth = 20;  // Number of DOM levels to capture
```

To modify:
1. Right-click on the chart where EA is attached
2. Select **Expert Advisors** → **Properties**
3. Navigate to the **Inputs** tab
4. Adjust `DOM_Depth` (range: 1-20)
5. Click **OK**

### Setting Up DOM (Depth of Market) Access

DOM access requires specific broker permissions:

1. **Verify Exchange Connection:**
   - DOM only works with exchange-connected accounts
   - Market makers/ECN brokers typically provide DOM
   - Standard retail accounts may not have access

2. **Check Symbol Properties:**
   ```mql5
   // In MT5, go to:
   // Market Watch → Right-click symbol → Specification
   // Look for "Market Depth" or "Level 2" support
   ```

3. **Test DOM Subscription:**
   ```mql5
   // The EA will attempt to subscribe automatically
   if (!MarketBookAdd(Symbol())) {
       // ERR_503 will be triggered if this fails
       Print("DOM subscription failed - check broker permissions");
   }
   ```

### Broker Permissions Needed

Ensure your broker account has:

| Permission | Required For | Check Method |
|------------|--------------|--------------|
| Real-time quotes | Tick streaming | Market Watch shows live prices |
| Level 2 DOM data | DOM streaming | Depth of Market window opens |
| Automated trading | EA execution | Tools → Options → Expert Advisors → Allow automated trading |
| DLL imports | Bridge loading | Tools → Options → Expert Advisors → Allow DLL imports |

**Enable DLL Imports in MT5:**
1. Tools → Options (Ctrl+O)
2. Expert Advisors tab
3. Check **Allow DLL imports**
4. Check **Allow WebRequest for listed URL** (if needed for external logging)
5. Click OK

---

## Loading the EA

### Step 1: Start RouDi Daemon

**CRITICAL:** Start RouDi before loading the EA in MT5.

```powershell
# Navigate to iceoryx build directory
cd third_party/iceoryx/build

# Start RouDi daemon
.\iox-roudi.exe
```

You should see output similar to:
```
2024-01-15 10:30:00.000 [Info]: RouDi is ready
```

**Keep this terminal window open.** RouDi must run continuously while the EA is active.

### Step 2: Compile the EA in MetaEditor

1. Open **MetaEditor 5** (from MT5, click Tools → MetaEditor)
2. Navigate to: `MQL5/Experts/TitanBridge/HighPerformanceBridgeEA.mq5`
3. Press **F7** or click **Compile**
4. Verify no errors in the "Errors" tab
5. Close MetaEditor

### Step 3: Attach EA to Chart

1. In MT5, open the **Navigator** window (Ctrl+N)
2. Expand **Expert Advisors**
3. Find **TitanBridge** folder
4. Drag **HighPerformanceBridgeEA** to your desired chart

### Step 4: Configure EA Properties

When attaching the EA, a properties dialog appears:

**Common Tab:**
- [x] Allow Alerts
- [x] Allow Live Trading
- [x] Allow DLL imports (REQUIRED)
- [x] Allow import of external experts

**Inputs Tab:**
- `DOM_Depth`: Set to desired depth (default: 20)

Click **OK** to attach.

### Step 5: Verify EA is Running

Successful initialization indicators:

1. **Smiley face** appears in top-right corner of chart
2. **Journal tab** shows:
   ```
   Starting High Performance Bridge EA...
   Session UUID: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
   ```
3. **No error dialogs** appear
4. **Experts tab** shows periodic heartbeat messages:
   ```
   Heartbeat broadcast for symbol: <symbol_id>
   ```

### Checking Logs

**MT5 Internal Logs:**
1. View → Strategy Tester (for backtest logs)
2. View → Journal (for terminal logs)
3. View → Experts (for EA-specific logs)

**Titan Bridge Logs:**
- Log file: `titan_bridge.log` (in working directory)
- Format: Structured JSON

Example log entry:
```json
{
  "timestamp": "2024-01-15T10:30:00.000Z",
  "level": "INFO",
  "module": "lib",
  "function": "bridge_init",
  "message": "Titan Protocol Bridge initialized successfully."
}
```

**Verify DLL Calls Are Working:**

Check the Experts tab for these messages:
```
Initializing Titan Bridge DLL...
Titan Protocol Bridge initializing...
Titan Protocol Bridge initialized successfully.
```

If you see:
```
Failed to initialize Titan Bridge DLL.
```
Check that RouDi is running and the DLL path is correct.

---

## Testing Procedures

### Test 1: Verify Tick Streaming

**Purpose:** Confirm ticks are being captured and published

**Steps:**
1. Attach EA to a chart with an active symbol (e.g., EURUSD)
2. Wait for market activity (or trigger a tick manually)
3. Check MT5 Experts tab for tick sequence messages
4. Verify log file shows tick processing

**Expected Output:**
```
Experts: Tick sequence #1 published
Experts: Tick sequence #2 published
...
```

**Verify in Logs:**
```json
{
  "timestamp": "2024-01-15T10:30:01.123Z",
  "level": "INFO",
  "module": "lib",
  "function": "bridge_push_tick",
  "message": "Tick processed - Symbol: <id>, Sequence: 1"
}
```

### Test 2: Verify DOM Streaming

**Purpose:** Confirm Level 2 market depth is being captured

**Steps:**
1. Ensure DOM is available for the symbol
2. Open the DOM window in MT5 (right-click symbol → Depth of Market)
3. Observe DOM updates in real-time
4. Check EA logs for book events

**Expected Behavior:**
- EA should receive `OnBookEvent` callbacks
- DOM data should be published to Iceoryx
- No throttling messages (unless under heavy load)

**Check for Backpressure:**
If you see:
```
Received BACKPRESSURE_WARNING from rust. Throttling DOM.
```
This indicates high load - the circuit breaker is functioning correctly.

### Test 3: Verify Circuit Breakers

**Purpose:** Confirm TPS limiting and auto-recovery work

**Steps:**
1. Set a low `max_tps` in `config/system.json` (e.g., 10)
2. Restart EA
3. Observe behavior under normal market conditions
4. Check for circuit breaker messages

**Expected Output:**
```
ERR_201: Data circuit breaker tripped - TPS exceeded
Waiting for cooldown period...
Circuit breaker reset - resuming normal operation
```

**Restore Configuration:**
```json
{
  "max_tps": 10000,
  "trip_duration_ms": 5000
}
```

### Test 4: Check Logs for Errors

**Purpose:** Verify no critical errors are occurring

**Checklist:**
- [ ] No ERR_001 (RouDi not found)
- [ ] No ERR_002 (Shared memory locked)
- [ ] No ERR_004 (Config file error)
- [ ] No ERR_100 (Null pointer)
- [ ] No ERR_101 (CRC validation failed)

**Command to check for errors:**
```powershell
# Search log file for CRITICAL entries
Select-String -Path "titan_bridge.log" -Pattern "CRITICAL"
```

**Expected Result:** No critical errors during normal operation

### Test 5: Verify Protobuf Output

**Purpose:** Confirm data is serialized correctly for consumers

**Prerequisites:**
- Protobuf consumer application available
- Iceoryx subscriber implemented

**Verification Steps:**
1. Start a test consumer (if available)
2. Subscribe to Titan topics:
   - Service: "Titan"
   - Instance: "MarketData"
   - Topics: "TickStream", "DepthOfMarket"

3. Verify received data structure matches `schemas/titan.proto`

**Example Consumer Output:**
```
Received Tick:
  symbol_id: 12345
  timestamp_ms: 1705315800123
  bid: 1.08560
  ask: 1.08562
  volume: 100
```

**Manual Verification:**
If no consumer is available, verify the DLL exports:
```rust
// In src/lib.rs, these functions should return true
bridge_push_tick() -> bool
bridge_push_book() -> bool
bridge_push_metadata() -> bool
```

---

## Troubleshooting

### Common Errors and Solutions

#### ERR_001: Iceoryx RouDi Not Found

**Symptoms:**
```
CRITICAL HALT [ERR_001_ICEORYX_ROUDI_NOT_FOUND]: 
  HighPerformanceBridgeEA::OnInit -> Failed to attach to iceoryx memory pool.
```

**Cause:** RouDi daemon is not running

**Solution:**
1. Start RouDi:
   ```powershell
   cd third_party/iceoryx/build
   .\iox-roudi.exe
   ```
2. Verify it's running:
   ```powershell
   Get-Process | Where-Object { $_.ProcessName -like "*roudi*" }
   ```
3. Restart MT5 EA

#### ERR_004: Config File Error

**Symptoms:**
```
CRITICAL HALT [ERR_004_CONFIG_FILE_ERROR]: 
  config::load -> CRITICAL: Missing config/system.json
```

**Cause:** Configuration file missing or malformed

**Solution:**
1. Verify file exists: `config/system.json`
2. Check JSON syntax:
   ```json
   {
     "max_tps": 10000,
     "trip_duration_ms": 5000,
     "max_slippage_points": 5.0,
     "iceoryx_threshold_percent": 80.0,
     "app_name": "TitanBridge"
   }
   ```
3. Ensure MT5 working directory contains the config folder

#### ERR_100: Null Pointer Dereference

**Symptoms:**
```
CRITICAL HALT [ERR_100_NULL_POINTER_DEREFERENCE]: 
  lib::bridge_push_tick -> Received null pointer for BinaryTick
```

**Cause:** Memory corruption in MQL5

**Solution:**
1. Restart MT5 Terminal completely
2. Recompile the EA in MetaEditor
3. Check struct alignment in `TitanProtocol.mqh` matches Rust definitions

#### ERR_101: CRC Validation Failed

**Symptoms:**
```
CRITICAL HALT [ERR_101_CRC_VALIDATION_FAILED]: 
  protocol::process_tick -> CRC validation failed (corrupt DMA transfer)
```

**Cause:** Struct mismatch between MQL5 and Rust

**Solution:**
1. Verify struct packing is consistent:
   ```mql5
   #pragma pack(push, 1)
   struct BinaryTick {
      // ... fields
   };
   #pragma pack(pop)
   ```
2. Check field sizes match exactly
3. Rebuild DLL and recompile EA

#### ERR_503: MT5 DOM Subscription Failed

**Symptoms:**
```
CRITICAL HALT [ERR_503_MT5_DOM_SUBSCRIPTION_FAILED]: 
  HighPerformanceBridgeEA::OnInit -> MarketBookAdd failed. Check broker permissions.
```

**Cause:** Broker doesn't provide DOM or permission denied

**Solution:**
1. Verify broker supports Level 2 data
2. Check account type (ECN/Pro accounts typically required)
3. Contact broker to enable DOM access
4. As workaround, disable DOM in EA (modify source)

### DLL Not Found

**Symptoms:**
```
Cannot load 'titan_bridge.dll'
```

**Solutions:**

1. **Verify DLL Path:**
   ```powershell
   Test-Path "$mt5Path\MQL5\Libraries\TitanBridge\titan_bridge.dll"
   ```

2. **Check DLL Architecture:**
   - Must be 64-bit (x64) to match MT5
   - Build with: `cargo build --release`

3. **Verify CRT Linking:**
   Ensure `.cargo/config.toml` contains:
   ```toml
   [target.x86_64-pc-windows-msvc]
   rustflags = ["-C", "target-feature=-crt-static"]
   ```

4. **Check Dependencies:**
   Use Dependency Walker or `dumpbin`:
   ```powershell
   dumpbin /dependents "target/release/titan_bridge.dll"
   ```

### Iceoryx Connection Issues

**Symptoms:**
- EA initializes but no data flows
- No errors in logs
- RouDi appears to be running

**Solutions:**

1. **Check RouDi Version Compatibility:**
   - Titan uses Iceoryx v2.95.8
   - Verify build matches this version

2. **Verify Shared Memory:**
   ```powershell
   # Check for Iceoryx shared memory segments
   Get-WmiObject Win32_Share | Where-Object { $_.Name -like "*iceoryx*" }
   ```

3. **Restart RouDi Clean:**
   ```powershell
   # Kill existing RouDi
   Get-Process iox-roudi | Stop-Process -Force
   
   # Clear shared memory (caution: affects all Iceoryx apps)
   # Then restart RouDi
   .\iox-roudi.exe
   ```

4. **Check Memory Limits:**
   RouDi requires sufficient shared memory. Ensure:
   - At least 1GB free RAM
   - Page file/virtual memory enabled

### Permission Problems

**Symptoms:**
- "Access Denied" errors
- Cannot write logs
- DLL fails to load

**Solutions:**

1. **Run MT5 as Administrator** (temporary test)
2. **Check Folder Permissions:**
   ```powershell
   # Grant full control to current user
   $path = "$mt5Path\MQL5"
   $user = $env:USERNAME
   icacls $path /grant "$user`:F" /T
   ```

3. **Antivirus Exclusions:**
   Add MT5 folder and Titan Bridge folder to antivirus exclusions

### High Memory Usage

**Symptoms:**
- MT5 memory consumption increases rapidly
- System becomes unresponsive

**Solutions:**

1. **Reduce DOM Depth:**
   ```mql5
   input int DOM_Depth = 5;  // Instead of 20
   ```

2. **Lower TPS Threshold:**
   ```json
   {
     "max_tps": 5000,
     "trip_duration_ms": 10000
   }
   ```

3. **Monitor Memory:**
   Check `titan_bridge.log` for memory warnings

---

## Verification Checklist

Before going live with the Titan Bridge, verify all items below:

### Pre-Deployment

- [ ] MetaTrader 5 is updated to latest version
- [ ] Broker account has DOM/Level 2 access enabled
- [ ] DLL compiled successfully with `cargo build --release`
- [ ] All files copied to correct MT5 directories
- [ ] `config/system.json` exists and is valid JSON
- [ ] Include paths in EA files are correct
- [ ] DLL import path in `IceoryxMemPool.mqh` is correct

### RouDi Daemon

- [ ] Iceoryx RouDi built successfully
- [ ] RouDi daemon is running before MT5 starts
- [ ] RouDi process visible in Task Manager
- [ ] No shared memory conflicts

### MT5 Configuration

- [ ] "Allow DLL imports" enabled in Options
- [ ] "Allow automated trading" enabled
- [ ] Symbol added to Market Watch
- [ ] DOM window opens and shows data
- [ ] EA compiles without errors in MetaEditor

### EA Loading

- [ ] EA attaches to chart successfully
- [ ] Smiley face indicator visible
- [ ] No error dialogs on initialization
- [ ] Session UUID printed in Experts tab
- [ ] Heartbeat messages appear every 5 seconds

### Data Streaming

- [ ] Tick sequence numbers incrementing
- [ ] DOM events triggering (if applicable)
- [ ] No critical errors in logs
- [ ] Circuit breaker functional (test with low TPS)
- [ ] CRC validation passing

### Log Verification

- [ ] `titan_bridge.log` file created
- [ ] Log contains initialization message
- [ ] No ERR_001 through ERR_004 errors
- [ ] No ERR_100 or ERR_101 errors
- [ ] JSON format valid in all log entries

### Performance

- [ ] MT5 remains responsive
- [ ] Memory usage stable
- [ ] No UI freezing during high activity
- [ ] CPU usage reasonable (< 50% on modern hardware)

### Backup Plans

- [ ] Know how to quickly remove EA (ExpertRemove)
- [ ] Have MT5 Terminal restart procedure ready
- [ ] Know RouDi restart procedure
- [ ] Configuration backups saved
- [ ] Log rotation strategy in place

### Final Sign-Off

- [ ] All tests in [Testing Procedures](#testing-procedures) passed
- [ ] Documentation reviewed
- [ ] Team members trained on operation
- [ ] Monitoring/alerting configured (if applicable)
- [ ] Rollback plan documented

---

## Quick Reference

### File Locations

| File | Source | Destination |
|------|--------|-------------|
| `titan_bridge.dll` | `target/release/` | `<MT5>/MQL5/Libraries/TitanBridge/` |
| `HighPerformanceBridgeEA.mq5` | `ea/` | `<MT5>/MQL5/Experts/TitanBridge/` |
| `*.mqh` includes | `ea/Include/` | `<MT5>/MQL5/Include/TitanBridge/` |
| `system.json` | `config/` | `<Titan_Project>/config/` |

### Key Commands

```powershell
# Build DLL
cargo build --release

# Start RouDi
.\third_party\iceoryx\build\iox-roudi.exe

# Run tests
cargo test

# Check logs
Get-Content titan_bridge.log -Tail 50

# Search for errors
Select-String -Path titan_bridge.log -Pattern "CRITICAL|ERROR"
```

### Error Codes Quick Reference

| Code | Meaning | Action |
|------|---------|--------|
| ERR_001 | RouDi not running | Start RouDi daemon |
| ERR_002 | Lock poisoned | Restart MT5 |
| ERR_004 | Config missing | Create config/system.json |
| ERR_100 | Null pointer | Restart MT5, check structs |
| ERR_101 | CRC failed | Check struct alignment |
| ERR_201 | Circuit tripped | Wait for auto-reset |
| ERR_503 | DOM not available | Check broker permissions |

---

## Support

For issues not covered in this guide:

1. Check the [Error Reference](error_reference.md)
2. Review the [Architecture Documentation](architecture.md)
3. Examine [Dependencies Guide](dependencies.md)
4. Contact: [ncdevshiv@gmail.com](mailto:ncdevshiv@gmail.com)
5. Support the project: [Buy Me a Coffee](https://buymeacoffee.com/ncdevshiv)

---

*Last Updated: 2024-01-15*  
*Version: 1.0*  
*Titan Protocol Architecture*