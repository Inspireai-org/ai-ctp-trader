# Bun 配置与最佳实践

## Bun 简介
- Bun 是一个快速的 JavaScript 运行时和包管理器
- 比 npm/yarn 提供更快的安装和执行速度
- 内置测试运行器、打包器和转译器

## 配置文件 (.bunfig.toml)
```toml
[install]
# 使用硬链接策略提高安装速度
strategy = "hardlink"
# 自动安装 peer dependencies
auto = true
# 启用缓存
cache = true

[run]
# 使用 zsh 作为默认 shell
shell = "zsh"

[test]
# 测试预加载文件
preload = ["./test-setup.ts"]
```

## 常用命令
```bash
# 安装依赖
bun install

# 运行脚本
bun run dev
bun run build
bun run test

# Tauri 相关命令
bun run tauri:dev
bun run tauri:build
bun run tauri:android:dev
bun run tauri:ios:dev

# 添加依赖
bun add <package>
bun add -d <package>  # 开发依赖

# 移除依赖
bun remove <package>

# 审计依赖
bun audit

# 更新依赖
bun update
```

## 性能优化
- 使用 `bun.lock` 锁定依赖版本
- 配置 `.bunfig.toml` 优化安装策略
- 利用 Bun 的内置缓存机制
- 使用 Bun 的快速模块解析

## 测试配置
- 使用 Bun 内置测试运行器替代 Jest
- 配置测试预加载文件
- 支持 TypeScript 无需额外配置
- 内置代码覆盖率报告

## 与 Tauri 集成
- 在 `tauri.conf.json` 中使用 bun 命令
- 配置 `beforeDevCommand` 和 `beforeBuildCommand`
- 支持热重载和快速构建
- 移动端开发支持

## 迁移指南
1. 删除 `node_modules` 和 `package-lock.json`/`yarn.lock`
2. 运行 `bun install` 安装依赖
3. 更新 `package.json` 脚本
4. 配置 `.bunfig.toml`
5. 更新 CI/CD 流程使用 bun 命令

## 故障排除
- 如果遇到兼容性问题，检查包的 Bun 支持情况
- 使用 `bun --bun` 强制使用 Bun 运行时
- 查看 Bun 官方文档获取最新信息
- 在 GitHub Issues 中搜索相关问题