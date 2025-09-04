# CTP 交易组件

基于 Rust 的 CTP (Comprehensive Transaction Platform) 交易组件，提供类型安全、内存安全的期货交易接口。

## 功能特性

- ✅ **类型安全**: 使用 Rust 的类型系统确保 API 调用的正确性
- ✅ **内存安全**: 通过 Rust 的所有权系统防止内存泄漏和悬垂指针
- ✅ **异步支持**: 基于 tokio 的异步编程模型
- ✅ **事件驱动**: 完整的事件处理和回调机制
- ✅ **错误处理**: 统一的错误类型和处理策略
- ✅ **配置管理**: 灵活的配置文件和环境变量支持
- ✅ **跨平台**: 支持 Windows、Linux、macOS 平台

## 模块结构

```
src/ctp/
├── mod.rs              # 模块入口和公共接口
├── client.rs           # CTP 客户端主要实现
├── config.rs           # 配置数据结构
├── config_manager.rs   # 配置管理器
├── error.rs            # 错误类型定义
├── events.rs           # 事件系统
├── models.rs           # 数据模型定义
├── ffi.rs              # FFI 绑定和库管理
├── tests.rs            # 单元测试
└── README.md           # 本文档
```

## 快速开始

### 1. 基本使用

```rust
use crate::ctp::{CtpClient, CtpConfig, LoginCredentials};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建配置
    let mut config = CtpConfig::default();
    config.investor_id = "your_investor_id".to_string();
    config.password = "your_password".to_string();
    
    // 创建客户端
    let mut client = CtpClient::new(config).await?;
    
    // 连接服务器
    client.connect().await?;
    
    // 用户登录
    let credentials = LoginCredentials {
        broker_id: "9999".to_string(),
        user_id: "your_investor_id".to_string(),
        password: "your_password".to_string(),
        app_id: "simnow_client_test".to_string(),
        auth_code: "0000000000000000".to_string(),
    };
    
    let login_response = client.login(credentials).await?;
    println!("登录成功: {:?}", login_response);
    
    // 订阅行情
    client.subscribe_market_data(&["rb2401".to_string()]).await?;
    
    // 查询账户信息
    let account_info = client.query_account().await?;
    println!("账户信息: {:?}", account_info);
    
    Ok(())
}
```

### 2. 事件处理

```rust
use crate::ctp::{CtpClient, CtpEvent, EventListener};

struct MyEventListener;

impl EventListener for MyEventListener {
    fn on_market_data(&self, tick: &MarketDataTick) {
        println!("收到行情: {} 价格: {}", tick.instrument_id, tick.last_price);
    }
    
    fn on_order_update(&self, order: &OrderStatus) {
        println!("订单更新: {} 状态: {:?}", order.order_id, order.status);
    }
    
    fn on_error(&self, error: &CtpError) {
        eprintln!("CTP 错误: {}", error);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = CtpClient::new(config).await?;
    let mut event_handler = client.event_handler();
    
    // 启动事件处理循环
    tokio::spawn(async move {
        while let Some(event) = event_handler.next_event().await {
            match event {
                CtpEvent::MarketData(tick) => {
                    println!("行情更新: {}", tick.instrument_id);
                }
                CtpEvent::OrderUpdate(order) => {
                    println!("订单更新: {}", order.order_id);
                }
                CtpEvent::Error(error) => {
                    eprintln!("错误: {}", error);
                }
                _ => {}
            }
        }
    });
    
    // 其他业务逻辑...
    
    Ok(())
}
```

### 3. 配置管理

```rust
use crate::ctp::{ConfigManager, ExtendedCtpConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从文件加载配置
    let config = ConfigManager::load_from_file("ctp_config.toml").await?;
    
    // 从环境变量加载配置
    let env_config = ConfigManager::load_from_env()?;
    
    // 合并配置（环境变量优先）
    let final_config = ConfigManager::merge_configs(config.ctp, env_config);
    
    // 创建客户端
    let client = CtpClient::new(final_config).await?;
    
    Ok(())
}
```

## 配置说明

### 配置文件格式 (TOML)

```toml
[connection]
md_front_addr = "tcp://180.168.146.187:10131"
trader_front_addr = "tcp://180.168.146.187:10130"

[credentials]
broker_id = "9999"
investor_id = "your_investor_id"
password = "your_password"
app_id = "simnow_client_test"
auth_code = "0000000000000000"

[settings]
flow_path = "./ctp_flow/"
timeout_secs = 30
reconnect_interval_secs = 5
max_reconnect_attempts = 3

[logging]
level = "info"
file_path = "./logs/ctp.log"
console = true

[environment]
env_type = "development"
simulation_mode = true
```

### 环境变量

```bash
export CTP_BROKER_ID="9999"
export CTP_INVESTOR_ID="your_investor_id"
export CTP_PASSWORD="your_password"
export CTP_MD_FRONT_ADDR="tcp://180.168.146.187:10131"
export CTP_TRADER_FRONT_ADDR="tcp://180.168.146.187:10130"
export CTP_LIB_PATH="/path/to/ctp/libs"
```

## API 参考

### 核心类型

#### CtpClient
主要的 CTP 客户端类，提供所有交易功能。

**方法**:
- `new(config: CtpConfig) -> Result<Self, CtpError>`
- `connect() -> Result<(), CtpError>`
- `login(credentials: LoginCredentials) -> Result<LoginResponse, CtpError>`
- `subscribe_market_data(instruments: &[String]) -> Result<(), CtpError>`
- `submit_order(order: OrderRequest) -> Result<String, CtpError>`
- `cancel_order(order_id: &str) -> Result<(), CtpError>`
- `query_account() -> Result<AccountInfo, CtpError>`
- `query_positions() -> Result<Vec<Position>, CtpError>`
- `disconnect()`

#### CtpConfig
CTP 连接配置结构。

**字段**:
- `md_front_addr: String` - 行情服务器地址
- `trader_front_addr: String` - 交易服务器地址
- `broker_id: String` - 经纪商代码
- `investor_id: String` - 投资者代码
- `password: String` - 密码
- `app_id: String` - 应用标识
- `auth_code: String` - 授权编码
- `flow_path: String` - 流文件路径
- `timeout_secs: u64` - 超时时间
- `reconnect_interval_secs: u64` - 重连间隔
- `max_reconnect_attempts: u32` - 最大重连次数

#### CtpEvent
CTP 事件枚举。

**变体**:
- `Connected` - 连接成功
- `Disconnected` - 连接断开
- `LoginSuccess(LoginResponse)` - 登录成功
- `LoginFailed(String)` - 登录失败
- `MarketData(MarketDataTick)` - 行情数据
- `OrderUpdate(OrderStatus)` - 订单更新
- `TradeUpdate(TradeRecord)` - 成交记录
- `AccountUpdate(AccountInfo)` - 账户更新
- `PositionUpdate(Vec<Position>)` - 持仓更新
- `Error(String)` - 错误事件

#### CtpError
CTP 错误类型。

**变体**:
- `ConnectionError(String)` - 连接错误
- `AuthenticationError(String)` - 认证错误
- `NetworkError(String)` - 网络错误
- `ApiError { code: i32, message: String }` - API 错误
- `ConversionError(String)` - 数据转换错误
- `ConfigError(String)` - 配置错误
- `FfiError(String)` - FFI 错误
- `TimeoutError` - 超时错误
- `LibraryLoadError(String)` - 库加载错误
- `Unknown(String)` - 未知错误

## 开发指南

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test ctp::tests

# 运行单个测试
cargo test test_ctp_config_default
```

### 调试模式

启用详细日志输出：

```bash
RUST_LOG=debug cargo test
```

### 添加新功能

1. 在相应的模块中添加新的结构体或函数
2. 更新相关的测试
3. 更新文档和示例
4. 确保所有测试通过

## 注意事项

1. **库文件依赖**: 使用前需要安装 CTP 动态库文件
2. **网络连接**: 确保能够访问 CTP 服务器
3. **账户权限**: 需要有效的期货交易账户
4. **线程安全**: 客户端实例不是线程安全的，需要在单个线程中使用
5. **资源管理**: 使用完毕后应该调用 `disconnect()` 方法

## 故障排除

### 常见问题

1. **库文件未找到**
   - 检查 CTP 库文件是否正确安装
   - 验证 `CTP_LIB_PATH` 环境变量设置

2. **连接失败**
   - 检查网络连接
   - 确认服务器地址和端口正确
   - 检查防火墙设置

3. **认证失败**
   - 确认账户信息正确
   - 检查经纪商代码
   - 验证授权码有效性

4. **编译错误**
   - 确保 Rust 版本符合要求
   - 检查依赖项版本
   - 运行 `cargo clean` 清理缓存

### 获取帮助

- 查看项目文档
- 检查 GitHub Issues
- 联系技术支持

## 许可证

本项目遵循 MIT 许可证。详情请参阅 LICENSE 文件。