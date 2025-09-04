# 开发指南

## 🛠️ 开发环境设置

### 必需工具
- **Bun**: v1.2.19+ (已安装 ✅)
- **Rust**: 最新稳定版 (已配置 ✅)
- **Tauri CLI**: v2.x (已安装 ✅)

### 开发命令
```bash
# 安装依赖
bun install

# 开发模式启动
bun run tauri:dev

# 类型检查
bun run type-check

# 运行测试
bun test

# 构建生产版本
bun run tauri:build
```

## 📁 项目结构

### 前端结构 (src/)
```
src/
├── components/     # 可复用组件
├── pages/         # 页面组件
├── hooks/         # 自定义 Hooks
├── stores/        # Zustand 状态管理
├── services/      # API 服务层
├── types/         # TypeScript 类型定义
├── utils/         # 工具函数
└── styles/        # 样式文件
```

### 后端结构 (src-tauri/src/)
```
src-tauri/src/
├── ctp/           # CTP 集成模块
│   ├── client.rs  # CTP 客户端
│   ├── spi/       # SPI 回调实现
│   ├── models.rs  # 数据模型
│   └── utils/     # 工具函数
├── commands/      # Tauri 命令
└── main.rs        # 主入口
```

## 🔧 开发规范

### TypeScript 规范
- 使用严格模式 (`strict: true`)
- 所有函数和变量必须有类型注解
- 使用 `exactOptionalPropertyTypes` 确保类型安全
- 优先使用接口而非类型别名

### Rust 规范
- 使用 `clippy` 进行代码检查
- 所有公共 API 必须有文档注释
- 错误处理使用 `Result<T, E>` 类型
- 异步操作使用 `tokio` 运行时

### Git 提交规范
```
feat: 新功能
fix: 修复bug
docs: 文档更新
style: 代码格式调整
refactor: 代码重构
test: 测试相关
chore: 构建工具或辅助工具的变动
```

## 🧪 测试策略

### 前端测试
- 使用 Bun 内置测试运行器
- 组件测试使用 React Testing Library
- 状态管理测试覆盖所有 store

### 后端测试
- 单元测试使用 Rust 内置测试框架
- 集成测试验证 CTP 功能
- 模拟测试环境避免真实交易

## 🚀 部署流程

### 开发环境
1. 确保所有依赖已安装
2. 运行 `bun run tauri:dev` 启动开发服务器
3. 使用浏览器开发者工具调试前端
4. 使用 `cargo test` 运行后端测试

### 生产环境
1. 运行完整测试套件
2. 执行 `bun run tauri:build` 构建应用
3. 测试生成的安装包
4. 部署到目标平台

## 🔍 调试技巧

### 前端调试
- 使用 React DevTools 检查组件状态
- 使用 Zustand DevTools 监控状态变化
- 使用浏览器 Network 面板检查 API 调用

### 后端调试
- 使用 `tracing` 库输出详细日志
- 使用 `cargo test -- --nocapture` 查看测试输出
- 使用 `RUST_LOG=debug` 环境变量启用调试日志

## 📚 相关资源

- [Tauri 官方文档](https://tauri.app/)
- [Bun 官方文档](https://bun.sh/)
- [ctp2rs 库文档](https://docs.rs/ctp2rs/)
- [React 官方文档](https://react.dev/)
- [Zustand 文档](https://zustand-demo.pmnd.rs/)