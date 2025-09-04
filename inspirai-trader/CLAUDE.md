# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Inspirai Trader** is a professional futures trading platform built with modern technologies for high-performance trading operations. The application provides a comprehensive trading interface for China's futures market through the CTP (Comprehensive Transaction Platform) API.

### Key Features
- Real-time market data streaming and visualization
- Professional K-line charts with technical indicators
- High-performance order execution and management
- Position monitoring and risk management
- Account fund tracking and settlement management
- Multi-environment support (SimNow, TTS, Production)

## Technology Stack

### Frontend
- **Framework**: React 19 + TypeScript 5.5
- **UI Components**: Ant Design 5.x
- **State Management**: Zustand
- **Charts**: ECharts for financial charts
- **Styling**: Tailwind CSS + CSS Modules
- **Build Tool**: Vite 6.0

### Backend
- **Core**: Rust + Tauri 2.1
- **Trading API**: CTP v6.7.9 via ctp2rs bindings
- **Async Runtime**: Tokio
- **Configuration**: TOML files
- **Logging**: tracing + tracing-subscriber
- **Serialization**: serde + serde_json

### Package Manager
- **Bun**: v1.1.40 for fast dependency management

## Development Commands

### Frontend Development
```bash
# Install dependencies
bun install

# Start development server with hot reload
bun run dev

# Run Tauri development mode (frontend + backend)
bun run tauri:dev

# Build production application
bun run tauri:build

# Type checking
bun run type-check

# Linting
bun run lint

# Format code
bun run format
```

### Backend Development
```bash
# Navigate to Rust backend
cd src-tauri

# Run all tests
cargo test

# Run specific test modules
cargo test ctp::tests
cargo test ctp::production_config_test
cargo test query_functionality_test

# Build release version
cargo build --release

# Run examples
cargo run --example query_demo
cargo run --example md_spi_demo

# Code formatting
cargo fmt

# Linting with clippy
cargo clippy -- -W clippy::pedantic

# Check compilation
cargo check
```

### Environment Setup
```bash
# Required for macOS - Set CTP library path
export CTP_LIB_PATH="../lib/macos/TraderapiMduserapi_6.7.7_CP_MacOS"

# Force CTP library linking in tests
export FORCE_CTP_LINK=1

# Enable debug logging
export RUST_LOG=debug

# Set development environment
export CTP_ENV=simnow  # or 'tts' or 'production'
```

## Project Architecture

### Directory Structure
```
inspirai-trader/
├── src/                    # React frontend source
│   ├── components/        # UI components
│   │   ├── layout/       # Layout components
│   │   ├── market/       # Market data components
│   │   ├── chart/        # Chart components
│   │   ├── trading/      # Trading panel components
│   │   └── common/       # Common components
│   ├── stores/           # Zustand state stores
│   ├── services/         # API service layer
│   ├── hooks/            # Custom React hooks
│   ├── types/            # TypeScript type definitions
│   ├── utils/            # Utility functions
│   └── styles/           # Global styles
│
├── src-tauri/            # Rust backend
│   ├── src/
│   │   ├── ctp/         # CTP trading component
│   │   │   ├── client.rs        # Main CTP client
│   │   │   ├── config.rs        # Configuration
│   │   │   ├── events.rs        # Event system
│   │   │   ├── spi/            # CTP SPI implementations
│   │   │   ├── models.rs       # Data models
│   │   │   └── services/       # Business services
│   │   └── lib.rs              # Tauri commands
│   └── config/                 # Environment configs
│
└── .kiro/                      # Project specifications
    ├── specs/                  # Feature specifications
    │   ├── futures-trading-ui/ # UI requirements
    │   └── ctp-trading-component/ # Backend requirements
    └── steering/               # Development guidelines
```

### Core Modules

#### CTP Trading Component (`src-tauri/src/ctp/`)
The heart of the trading system, implementing:

1. **Client Layer** (`client.rs`)
   - Thread-safe CTP client with connection management
   - Automatic reconnection and health monitoring
   - State machine for connection lifecycle

2. **Market Data Service** (`market_data_service.rs`)
   - Real-time market data streaming
   - Subscription management with priority queue
   - Data filtering and caching

3. **Trading Service** (`trading_service.rs`)
   - Order submission and management
   - Position tracking and P&L calculation
   - Risk monitoring

4. **Query Service** (`query_service.rs`)
   - Unified query interface for all CTP data
   - Caching layer for performance
   - Batch query support

5. **Event System** (`events.rs`)
   - Asynchronous event-driven architecture
   - Multi-listener support
   - Error propagation

#### Frontend Components (`src/components/`)
Professional trading UI implementing:

1. **TradingLayout**: Main application layout with resizable panels
2. **MarketDataPanel**: Real-time quotes with sorting/filtering
3. **ChartPanel**: Professional K-line charts with indicators
4. **TradingPanel**: Order entry with risk calculations
5. **PositionPanel**: Position monitoring with P&L tracking

## Configuration

### Environment Configurations
Located in `src-tauri/config/`:

- **simnow.toml**: Development/testing environment
- **tts.toml**: Test trading system
- **production.toml**: Live trading (handle with care!)

### Example Configuration
```toml
[ctp]
environment = "SimNow"
broker_id = "9999"
app_id = "simnow_client"
auth_code = "0000000000000000"

[connection]
market_front = ["tcp://180.168.146.187:10131"]
trade_front = ["tcp://180.168.146.187:10130"]
timeout_ms = 5000
retry_count = 3

[account]
user_id = "your_user_id"
password = "your_password"
```

## Testing Strategy

### Unit Tests
- Colocated with source files
- Mock CTP responses for offline testing
- Focus on business logic validation

### Integration Tests
- `production_config_test.rs`: Configuration validation
- `simple_production_test.rs`: End-to-end workflow
- `query_functionality_test.rs`: Query service testing

### Running Tests
```bash
# All tests
cargo test

# Specific test file
cargo test --test query_functionality_test

# With output
cargo test -- --nocapture

# Single test function
cargo test test_query_service_creation
```

## Performance Guidelines

### Backend Optimization
- Market data processing target: <100ms latency
- Use `PerformanceMonitor` for profiling
- Implement caching for frequently accessed data
- Batch operations where possible

### Frontend Optimization
- Virtual scrolling for large lists
- React.memo for pure components
- Debounced market data updates
- Lazy loading for non-critical features

## Error Handling

### Backend Errors
- Unified `CtpError` type with error codes
- Automatic retry for recoverable errors
- Detailed logging with tracing

### Frontend Errors
- Error boundaries for graceful degradation
- User-friendly error messages
- Automatic reconnection handling

## Security Considerations

### Credentials
- Never commit credentials to version control
- Use environment variables for sensitive data
- Encrypt stored passwords

### Trading Safety
- Order validation before submission
- Risk limit checks
- Position limit enforcement
- Emergency stop functionality

## CTP Library Dependencies

The project requires CTP v6.7.9 libraries:

### Platform-specific Libraries
- **macOS**: Framework files in `../lib/macos/`
- **Linux**: `.so` files in `../lib/linux/`
- **Windows**: `.dll` files in `../lib/windows/`

### Installation
1. Download CTP libraries from official source
2. Place in appropriate directory
3. Set environment variable: `export CTP_LIB_PATH=path/to/libs`
4. Verify with: `cargo test ffi::check_ctp_libraries`

## Common Development Tasks

### Adding a New Trading Feature
1. Define data models in `src-tauri/src/ctp/models.rs`
2. Implement service logic in appropriate service file
3. Create Tauri command in `src-tauri/src/lib.rs`
4. Add TypeScript types in `src/types/`
5. Create React component in `src/components/`
6. Update state store in `src/stores/`

### Debugging Market Data
1. Enable debug logging: `export RUST_LOG=debug`
2. Check subscription status in logs
3. Verify connection state in client
4. Monitor event stream for data flow

### Performance Profiling
1. Use `PerformanceMonitor` in Rust code
2. React DevTools Profiler for frontend
3. Monitor memory usage with system tools
4. Check network latency with CTP metrics

## Troubleshooting

### Common Issues

#### CTP Library Not Found
```bash
# Set library path
export CTP_LIB_PATH="../lib/macos/TraderapiMduserapi_6.7.7_CP_MacOS"
export FORCE_CTP_LINK=1
```

#### Connection Timeout
- Check network connectivity
- Verify CTP server addresses
- Ensure firewall allows CTP ports

#### Market Data Not Updating
- Check subscription status
- Verify contract ID format
- Monitor event listeners

#### Build Failures
```bash
# Clean build
cargo clean
rm -rf target/
bun install --force
```

## Contributing Guidelines

1. **Code Style**
   - Follow Rust conventions (cargo fmt)
   - Use TypeScript strict mode
   - Maintain consistent naming

2. **Testing**
   - Write unit tests for new features
   - Update integration tests
   - Test in SimNow before production

3. **Documentation**
   - Update relevant .md files
   - Add inline documentation
   - Document API changes

4. **Performance**
   - Profile before optimization
   - Benchmark critical paths
   - Monitor memory usage

## Resources

- [CTP API Documentation](http://www.sfit.com.cn/DocumentDown/api/)
- [Tauri Documentation](https://tauri.app/v2/guide/)
- [React Documentation](https://react.dev/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Project Specs](./.kiro/specs/)

## Support

For issues or questions:
1. Check existing documentation
2. Review test examples
3. Enable debug logging
4. Create detailed issue report with logs