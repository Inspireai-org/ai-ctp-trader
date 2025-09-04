# CTP 交易组件环境配置指南

本文档详细说明如何配置 CTP (Comprehensive Transaction Platform) 交易组件的开发环境。

## 目录

1. [系统要求](#系统要求)
2. [快速开始](#快速开始)
3. [手动配置](#手动配置)
4. [CTP 库文件安装](#ctp-库文件安装)
5. [配置文件说明](#配置文件说明)
6. [环境变量](#环境变量)
7. [故障排除](#故障排除)

## 系统要求

### 基础环境
- **操作系统**: Windows 10+, macOS 10.15+, Ubuntu 18.04+
- **Rust**: 1.70.0 或更高版本
- **Bun**: 1.0.0 或更高版本
- **Node.js**: 18.0.0 或更高版本（Bun 的备选方案）

### 开发工具
- **Tauri CLI**: 2.0.0 或更高版本
- **Git**: 用于版本控制
- **VS Code**: 推荐的开发环境（可选）

## 快速开始

### 1. 自动化设置

我们提供了自动化设置脚本来简化环境配置过程：

#### Linux/macOS
```bash
cd inspirai-trader
chmod +x scripts/setup_ctp.sh
./scripts/setup_ctp.sh
```

#### Windows
```cmd
cd inspirai-trader
scripts\setup_ctp.bat
```

### 2. 验证安装

运行以下命令验证环境是否正确配置：

```bash
# 检查 Rust 环境
rustc --version
cargo --version

# 检查 Bun 环境
bun --version

# 检查 Tauri CLI
tauri --version

# 构建项目
bun run tauri:dev
```

## 手动配置

如果自动化脚本无法满足需求，可以按照以下步骤手动配置环境。

### 1. 安装 Rust

访问 [https://rustup.rs/](https://rustup.rs/) 下载并安装 Rust：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. 安装 Bun

访问 [https://bun.sh/](https://bun.sh/) 下载并安装 Bun：

```bash
curl -fsSL https://bun.sh/install | bash
```

### 3. 安装 Tauri CLI

```bash
cargo install tauri-cli
```

### 4. 创建目录结构

```bash
mkdir -p lib/{windows,linux,macos}
mkdir -p ctp_flow
mkdir -p logs
```

## CTP 库文件安装

### 1. 下载 CTP API

从上海期货信息技术有限公司官网下载 CTP API：

- **官网**: [http://www.sfit.com.cn/](http://www.sfit.com.cn/)
- **下载页面**: [http://www.sfit.com.cn/DocumentDown.htm](http://www.sfit.com.cn/DocumentDown.htm)

### 2. 安装库文件

根据你的操作系统，将相应的库文件复制到对应目录：

#### Windows
将以下文件复制到 `lib/windows/` 目录：
- `thostmduserapi_se.dll` (行情 API)
- `thosttraderapi_se.dll` (交易 API)

#### Linux
将以下文件复制到 `lib/linux/` 目录：
- `libthostmduserapi_se.so` (行情 API)
- `libthosttraderapi_se.so` (交易 API)

#### macOS
将以下文件复制到 `lib/macos/` 目录：
- `libthostmduserapi_se.dylib` (行情 API)
- `libthosttraderapi_se.dylib` (交易 API)

### 3. 验证库文件

运行应用程序时，系统会自动检查库文件是否存在。如果缺少必要的库文件，会在控制台显示相应的错误信息。

## 配置文件说明

### ctp_config.toml

主配置文件位于项目根目录的 `ctp_config.toml`，包含以下配置项：

```toml
[connection]
# 行情前置地址（SimNow 模拟环境）
md_front_addr = "tcp://180.168.146.187:10131"
# 交易前置地址（SimNow 模拟环境）
trader_front_addr = "tcp://180.168.146.187:10130"

[credentials]
# 经纪商代码
broker_id = "9999"
# 投资者代码（需要用户填写）
investor_id = ""
# 密码（需要用户填写）
password = ""
# 应用标识
app_id = "simnow_client_test"
# 授权编码
auth_code = "0000000000000000"

[settings]
# 流文件路径
flow_path = "./ctp_flow/"
# 超时时间（秒）
timeout_secs = 30
# 重连间隔（秒）
reconnect_interval_secs = 5
# 最大重连次数
max_reconnect_attempts = 3

[logging]
# 日志级别 (trace, debug, info, warn, error)
level = "info"
# 日志文件路径
file_path = "./logs/ctp.log"
# 是否启用控制台输出
console = true

[environment]
# 环境类型 (development, testing, production)
env_type = "development"
# 是否启用模拟模式
simulation_mode = true
```

### 配置项说明

#### 连接配置 (connection)
- `md_front_addr`: 行情服务器地址
- `trader_front_addr`: 交易服务器地址

#### 认证配置 (credentials)
- `broker_id`: 期货公司代码
- `investor_id`: 投资者账号
- `password`: 登录密码
- `app_id`: 应用程序标识
- `auth_code`: 授权码

#### 系统设置 (settings)
- `flow_path`: CTP 流文件存储路径
- `timeout_secs`: 网络操作超时时间
- `reconnect_interval_secs`: 重连间隔时间
- `max_reconnect_attempts`: 最大重连次数

## 环境变量

可以通过环境变量覆盖配置文件中的设置：

```bash
# CTP 库文件路径
export CTP_LIB_PATH="/path/to/ctp/libs"

# 连接配置
export CTP_MD_FRONT_ADDR="tcp://180.168.146.187:10131"
export CTP_TRADER_FRONT_ADDR="tcp://180.168.146.187:10130"

# 认证信息
export CTP_BROKER_ID="9999"
export CTP_INVESTOR_ID="your_investor_id"
export CTP_PASSWORD="your_password"
export CTP_APP_ID="simnow_client_test"
export CTP_AUTH_CODE="0000000000000000"

# 系统设置
export CTP_FLOW_PATH="./ctp_flow/"
export CTP_TIMEOUT_SECS="30"
```

## 故障排除

### 常见问题

#### 1. 库文件未找到

**错误信息**: `CTP 库文件 xxx 未找到`

**解决方案**:
- 检查库文件是否存在于正确的目录中
- 确认文件权限是否正确
- 验证环境变量 `CTP_LIB_PATH` 设置

#### 2. 库文件加载失败

**错误信息**: `库文件加载失败`

**解决方案**:
- 确认库文件与系统架构匹配（x86/x64）
- 检查是否缺少依赖库
- 在 Linux 上运行 `ldd` 检查依赖关系

#### 3. 连接超时

**错误信息**: `连接超时`

**解决方案**:
- 检查网络连接
- 确认服务器地址和端口正确
- 调整超时时间设置

#### 4. 认证失败

**错误信息**: `用户名或密码错误`

**解决方案**:
- 确认投资者账号和密码正确
- 检查经纪商代码是否匹配
- 验证授权码是否有效

### 调试技巧

#### 1. 启用详细日志

在配置文件中设置日志级别为 `debug` 或 `trace`：

```toml
[logging]
level = "debug"
```

#### 2. 检查日志文件

查看日志文件获取详细的错误信息：

```bash
tail -f logs/ctp.log
```

#### 3. 使用环境变量调试

临时设置环境变量进行调试：

```bash
RUST_LOG=debug bun run tauri:dev
```

### 获取帮助

如果遇到无法解决的问题，可以：

1. 查看项目的 GitHub Issues
2. 阅读 CTP API 官方文档
3. 联系技术支持团队

## 开发工作流

### 1. 开发模式

启动开发服务器：

```bash
bun run tauri:dev
```

### 2. 构建生产版本

构建应用程序：

```bash
bun run tauri:build
```

### 3. 运行测试

执行测试套件：

```bash
bun test
cargo test
```

### 4. 代码格式化

格式化代码：

```bash
bun run format
cargo fmt
```

## 部署说明

### 1. 生产环境配置

在生产环境中：
- 使用真实的 CTP 服务器地址
- 设置 `simulation_mode = false`
- 配置适当的日志级别
- 确保库文件的安全性

### 2. 安全注意事项

- 不要在代码中硬编码敏感信息
- 使用环境变量管理认证信息
- 定期更新 CTP 库文件
- 监控应用程序日志

---

更多详细信息请参考项目文档或联系开发团队。