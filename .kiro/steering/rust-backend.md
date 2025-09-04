# Rust 后端开发规范

## Tauri 命令设计
- 所有 Tauri 命令使用 async/await 模式
- 命令函数命名使用 snake_case
- 返回结果统一使用 Result<T, String> 类型
- 错误处理要详细，提供中文错误信息

## 数据库操作
- 使用 SQLite 作为本地数据库
- 使用 sqlx 或 rusqlite 进行数据库操作
- 数据库连接使用连接池管理
- 所有 SQL 查询使用预编译语句防止注入

## 网络通信
- HTTP 请求使用 reqwest 库
- WebSocket 连接使用 tokio-tungstenite
- 实现连接重试和错误恢复机制
- 网络超时设置合理的默认值

## 安全实践
- 敏感数据使用 AES 加密存储
- API 密钥和证书安全管理
- 输入验证和数据清理
- 日志记录不包含敏感信息

## 代码结构
```rust
// src-tauri/src/
├── main.rs              // 主入口
├── commands/            // Tauri 命令
│   ├── account.rs       // 账户相关命令
│   ├── trading.rs       // 交易相关命令
│   └── market.rs        // 行情相关命令
├── database/            // 数据库操作
│   ├── models.rs        // 数据模型
│   └── operations.rs    // 数据库操作
├── services/            // 业务服务
│   ├── market_service.rs // 行情服务
│   └── trading_service.rs // 交易服务
└── utils/               // 工具函数
    ├── crypto.rs        // 加密工具
    └── config.rs        // 配置管理
```

## 错误处理模式
```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(String),
    #[error("网络请求失败: {0}")]
    Network(String),
    #[error("认证失败: {0}")]
    Auth(String),
}
```