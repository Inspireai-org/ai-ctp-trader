# InspirAI Trader - 期货交易系统

基于 Tauri2 + React + TypeScript 构建的桌面期货交易应用程序。

## 技术栈

- **桌面框架**: Tauri 2.0
- **前端**: React 18 + TypeScript
- **包管理器**: Bun
- **构建工具**: Vite
- **后端**: Rust

## 开发环境设置

### 前置要求

1. 安装 [Rust](https://rustup.rs/)
2. 安装 [Bun](https://bun.sh/)
3. 安装 Tauri CLI: `cargo install tauri-cli`

### 快速开始

```bash
# 安装依赖
bun install

# 开发模式启动
bun run tauri:dev

# 构建应用
bun run tauri:build
```

### 移动端开发

```bash
# Android 初始化
bun run tauri:android:init

# Android 开发模式
bun run tauri:android:dev

# iOS 初始化
bun run tauri:ios:init

# iOS 开发模式
bun run tauri:ios:dev
```

## 推荐 IDE 设置

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
