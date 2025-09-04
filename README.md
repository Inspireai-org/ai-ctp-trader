# AI CTP Trader

基于 Tauri2 和 Rust 的智能期货交易系统，集成 CTP 接口实现专业的期货交易功能。

## 项目结构

```
ai-ctp-trader/
├── .kiro/                      # Kiro AI 助手配置和规范
│   ├── specs/                  # 项目规格说明
│   └── steering/               # 开发指导规范
├── inspirai-trader/            # 主要的 Tauri 应用
│   ├── src/                    # React 前端代码
│   ├── src-tauri/              # Rust 后端代码
│   │   └── src/ctp/           # CTP 集成模块
│   └── lib/                    # CTP 动态库文件
├── ctp-macos-demo/            # CTP macOS 示例程序
└── ctp2rs/                    # CTP Rust 绑定库 (submodule)
```

## 核心功能

- **实时行情**: 基于 CTP 接口的实时期货行情数据
- **交易执行**: 支持期货合约的买卖、开平仓操作
- **账户管理**: 资金查询、持仓管理、风险控制
- **策略交易**: 可扩展的量化交易策略框架
- **跨平台**: 支持 Windows、macOS、Linux 桌面端

## 技术栈

- **前端**: React 18 + TypeScript + Ant Design
- **后端**: Rust + Tauri2
- **包管理**: Bun (高性能 JavaScript 运行时)
- **数据库**: SQLite
- **CTP 集成**: ctp2rs 库
- **构建工具**: Vite + Cargo

## 快速开始

### 环境要求

- Rust 1.70+
- Node.js 18+ 或 Bun
- CTP 6.7.7+ 动态库

### 安装依赖

```bash
# 克隆项目（包含 submodule）
git clone --recursive https://github.com/your-org/ai-ctp-trader.git
cd ai-ctp-trader

# 安装前端依赖
cd inspirai-trader
bun install

# 安装 Tauri CLI
cargo install tauri-cli
```

### 开发模式

```bash
# 启动开发服务器
cd inspirai-trader
bun run tauri:dev
```

### 生产构建

```bash
# 构建应用
cd inspirai-trader
bun run tauri:build
```

## CTP 配置

项目支持多种 CTP 环境：

- **SimNow**: 上期技术模拟环境
- **OpenCTP TTS**: 7x24小时测试环境  
- **生产环境**: 实盘交易环境

配置文件位于 `inspirai-trader/src-tauri/config/` 目录。

## 示例程序

`ctp-macos-demo` 目录包含一个独立的 CTP 接口示例程序，展示如何在 macOS 上使用 ctp2rs 库：

```bash
cd ctp-macos-demo
cargo run --release
```

## 开发规范

项目遵循严格的开发规范，详见 `.kiro/steering/` 目录：

- [CTP 集成标准](/.kiro/steering/ctp-integration-standards.md)
- [Rust 后端规范](/.kiro/steering/rust-backend.md)
- [React 开发规范](/.kiro/steering/react-development.md)
- [Tauri 架构规范](/.kiro/steering/tauri-architecture.md)

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request。请确保遵循项目的开发规范和代码风格。