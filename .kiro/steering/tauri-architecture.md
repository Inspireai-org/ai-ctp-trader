# Tauri2 架构规范

## 项目结构
- 使用 Tauri2 最新版本作为桌面应用框架
- 前端使用 React.js + TypeScript
- 后端使用 Rust 编写 Tauri 命令
- 数据存储优先使用 SQLite，配合 Tauri 的数据库插件
- 包管理使用 Bun 替代 npm/yarn，提供更快的安装和运行速度
- 支持跨平台部署（Windows、macOS、Linux、Android、iOS）

## 技术栈要求
- **包管理器**: Bun (高性能 JavaScript 运行时和包管理器)
- **前端框架**: React 18+ with TypeScript
- **状态管理**: 使用 Zustand 或 Redux Toolkit
- **UI 组件库**: Ant Design 或 Material-UI
- **图表库**: TradingView Charting Library 或 ECharts
- **HTTP 客户端**: Axios 或 Fetch API
- **WebSocket**: 原生 WebSocket 或 Socket.io-client

## Tauri 配置原则
- 启用必要的 API 权限（文件系统、网络、通知等）
- 配置适当的 CSP（内容安全策略）
- 使用 Tauri 的安全上下文进行敏感操作
- 合理配置窗口属性和系统托盘
- 使用 bun 作为构建命令（beforeDevCommand 和 beforeBuildCommand）
- 配置移动端支持（Android 和 iOS）

## 代码组织
- `/src-tauri/` - Rust 后端代码
- `/src/` - React 前端代码
- `/src/components/` - 可复用组件
- `/src/pages/` - 页面组件
- `/src/hooks/` - 自定义 React Hooks
- `/src/stores/` - 状态管理
- `/src/services/` - API 服务层
- `/src/types/` - TypeScript 类型定义
- `/.bunfig.toml` - Bun 配置文件
- `/bun.lock` - Bun 锁文件（替代 package-lock.json）

## 开发工作流
- 使用 `bun install` 安装依赖
- 使用 `bun run tauri:dev` 启动开发服务器
- 使用 `bun run tauri:build` 构建生产版本
- 移动端开发使用对应的 `tauri:android:*` 和 `tauri:ios:*` 命令

## 性能优化
- Bun 提供更快的包安装和脚本执行速度
- 使用 Vite 作为构建工具，支持热重载和快速构建
- Tauri2 的新架构提供更好的性能和更小的包体积