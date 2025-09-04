# 测试与部署规范

## 测试策略

### 前端测试
- **单元测试**: 使用 Bun 内置测试运行器 + React Testing Library
- **组件测试**: 测试组件渲染和用户交互
- **Hook 测试**: 使用 @testing-library/react-hooks
- **集成测试**: 测试页面级别的功能流程
- **测试运行**: 使用 `bun test` 命令，享受更快的测试执行速度

### 后端测试
- **单元测试**: 使用 Rust 内置测试框架
- **集成测试**: 测试 Tauri 命令的完整流程
- **数据库测试**: 使用内存数据库进行测试
- **API 测试**: 模拟网络请求和响应

### 测试覆盖率
- 代码覆盖率目标 > 80%
- 关键业务逻辑覆盖率 > 95%
- 定期运行测试并生成覆盖率报告

## 构建与部署

### 开发环境
```bash
# 安装依赖
bun install
cargo install tauri-cli

# 开发模式启动
bun run tauri:dev

# 移动端开发
bun run tauri:android:init  # Android 初始化
bun run tauri:android:dev   # Android 开发
bun run tauri:ios:init      # iOS 初始化
bun run tauri:ios:dev       # iOS 开发
```

### 生产构建
```bash
# 构建应用
bun run tauri:build

# 生成安装包
# Windows: .msi 和 .exe
# macOS: .dmg 和 .app
# Linux: .deb 和 .AppImage

# 移动端构建
bun run tauri android build  # Android APK/AAB
bun run tauri ios build      # iOS IPA
```

### 版本管理
- 使用语义化版本号 (Semantic Versioning)
- 每个版本都要有详细的更新日志
- 支持增量更新和回滚机制

### 安全检查
- 依赖漏洞扫描 (bun audit, cargo audit)
- 代码静态分析 (ESLint, Clippy)
- 安全配置检查 (Tauri 权限配置)

## 性能监控
- 应用启动时间监控
- 内存使用情况跟踪
- 网络请求性能分析
- 用户操作响应时间统计

## 错误处理与日志
- 统一的错误处理机制
- 详细的日志记录（开发/生产环境分级）
- 崩溃报告收集和分析
- 用户反馈收集渠道

## 发布流程
1. 代码审查和测试通过
2. 版本号更新和标签创建
3. 自动化构建和打包
4. 安装包签名和验证
5. 发布到分发渠道
6. 用户通知和文档更新