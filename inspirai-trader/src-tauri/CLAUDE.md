# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an InspirAI Trader CTP (Comprehensive Transaction Platform) trading client built with Tauri 2 + React + TypeScript for futures trading. The project uses Rust for the backend trading logic and React for the frontend UI.

## Development Commands

### Frontend (from project root)
```bash
# Install dependencies
bun install

# Run development server
bun run tauri:dev

# Build production app
bun run tauri:build

# Linting and formatting
bun run lint            # ESLint check
bun run format          # Prettier format
bun run type-check      # TypeScript check
```

### Backend (from src-tauri/)
```bash
# Run all tests
cargo test

# Run specific test module
cargo test ctp::tests
cargo test ctp::production_config_test
cargo test ctp::simple_production_test

# Run single test
cargo test test_ctp_config_default

# Run tests with debug output
RUST_LOG=debug cargo test

# Build release version
cargo build --release

# Run examples
cargo run --example basic_usage
cargo run --example md_spi_demo
cargo run --example production_config_demo

# Format code
cargo fmt

# Lint with clippy
cargo clippy
```

### Environment Setup
```bash
# Set CTP library path (required for macOS)
export CTP_LIB_PATH="../lib/macos/TraderapiMduserapi_6.7.7_CP_MacOS"

# Force CTP link in tests
export FORCE_CTP_LINK=1
```

## Architecture

### Core CTP Module Structure

The CTP trading component (`src/ctp/`) implements a layered architecture:

1. **Client Layer** (`client.rs`)
   - `CtpClient`: Main client managing connections and state
   - Handles connection lifecycle, health checks, and retry logic
   - Thread-safe state management using `Arc<Mutex<>>`

2. **Configuration Layer** (`config.rs`, `config_manager.rs`)
   - Multi-environment support: SimNow (simulation), TTS (test), Production
   - TOML configuration files in `config/` directory
   - Dynamic library path detection for cross-platform support

3. **Market Data Layer** 
   - `MarketDataManager`: Caches and filters market data
   - `SubscriptionManager`: Manages subscription queue with priority and retry
   - `MarketDataService`: Unified service interface with lifecycle management
   - `MdSpiImpl`: Handles CTP market data callbacks

4. **Event System** (`events.rs`)
   - Event-driven architecture with `CtpEvent` enum
   - Async event handling via tokio channels
   - Event listeners for market data, order updates, and errors

5. **Error Handling** (`error.rs`)
   - Unified `CtpError` type with detailed error codes
   - Automatic retry detection for recoverable errors

### Key Design Patterns

- **SPI Callback Pattern**: The `spi/` module implements CTP's callback interfaces for receiving market data and trading events
- **Filter Chain**: Extensible `MarketDataFilter` trait for data processing pipelines
- **Priority Queue**: Subscription requests handled by priority (Urgent > High > Normal > Low)
- **State Machine**: Client states (Disconnected → Connecting → Connected → LoggingIn → LoggedIn)

### Data Flow

1. CTP API callbacks → SPI implementations (`md_spi.rs`)
2. Data conversion (GB18030 → UTF-8) and validation
3. Event generation and dispatch via channels
4. Data filtering and caching in managers
5. Service layer provides unified async interface

### Configuration Environments

Each environment has specific settings in `config/`:
- **simnow.toml**: Development/testing with SimNow servers
- **tts.toml**: Test environment for strategy validation  
- **production.toml**: Live trading configuration (handle with care)

### Critical Files

- `src/ctp/ffi.rs`: FFI bindings to CTP C++ libraries via ctp2rs crate
- `src/ctp/utils/encoding.rs`: GB18030 ↔ UTF-8 conversion for Chinese markets
- `build.rs`: Configures CTP library linking for different platforms

## Testing Strategy

- Unit tests are colocated with implementation files
- Integration tests in `production_config_test.rs` and `simple_production_test.rs`
- Examples demonstrate complete workflows in `examples/`
- Mock CTP responses for testing without live connection

## Performance Considerations

- Market data processing target: <100ms latency
- Use `PerformanceMonitor` for profiling critical paths
- Caching layer reduces redundant data processing
- Async/await for non-blocking I/O operations

## CTP Library Dependencies

The project depends on CTP v6.7.9 libraries which must be installed separately:
- macOS: Framework files in `../lib/macos/`
- Linux: `.so` files in `../lib/linux/`
- Windows: `.dll` files in `../lib/windows/`

Use `ctp2rs` crate v0.1.7 which provides safe Rust bindings to the CTP API.