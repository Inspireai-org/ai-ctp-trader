# CTP macOS Demo

基于 ctp2rs 的 CTP 接口 macOS 示例程序，展示如何在 macOS 平台上使用 CTP API。

## 功能特性

- 支持 macOS 平台 (Apple Silicon & Intel)
- 使用 CTP 6.7.7 版本动态库
- 支持 SimNow 和 OpenCTP TTS 环境
- 实现行情订阅和接收功能
- 命令行参数配置

## 环境要求

- macOS 系统
- Rust 1.70+
- CTP 6.7.7 动态库 (项目已包含)

## 项目结构

```
ctp-macos-demo/
├── Cargo.toml          # 项目配置
├── src/
│   ├── main.rs        # 主程序入口
│   └── mdapi.rs       # 行情 API 实现
└── README.md          # 本文件
```

## 使用方法

### 1. 编译项目

```bash
cd ctp-macos-demo
cargo build --release
```

### 2. 运行程序

#### 使用 OpenCTP TTS 环境（默认）

```bash
# 使用内置测试账号
cargo run --release

# 或使用自定义账号
cargo run --release -- -u YOUR_USER_ID -p YOUR_PASSWORD
```

#### 使用 SimNow 环境

```bash
cargo run --release -- -e sim -u YOUR_USER_ID -p YOUR_PASSWORD
```

### 3. 命令行参数

- `-e, --environment <ENV>`: 运行环境 (tts 或 sim)，默认为 tts
- `-u, --user-id <USER_ID>`: 用户ID
- `-p, --password <PASSWORD>`: 密码
- `-b, --broker-id <BROKER_ID>`: 经纪商ID，默认为 9999

也支持通过环境变量设置：
- `CTP_USER_ID`
- `CTP_PASSWORD`
- `CTP_BROKER_ID`

## 动态库配置

项目使用的 CTP 动态库位于：
```
../inspirai-trader/lib/macos/TraderapiMduserapi_6.7.7_CP_MacOS/
├── thostmduserapi_se.framework/   # 行情 API
└── thosttraderapi_se.framework/   # 交易 API
```

## 测试环境

### OpenCTP TTS (7x24小时)
- 行情前置: tcp://121.37.80.177:20004
- 交易前置: tcp://121.37.80.177:20002
- 测试账号: 209992 / CEE196Aa

### SimNow (标准时段)
- 行情前置: tcp://180.168.146.187:10211
- 交易前置: tcp://180.168.146.187:10201
- 需要自行注册账号: http://www.simnow.com.cn

## 注意事项

1. 确保动态库路径正确，相对路径基于可执行文件位置
2. 首次运行会在当前目录创建 `ctp_md_flow` 文件夹存储流文件
3. TTS 环境为 7x24 小时运行，适合测试
4. SimNow 环境仅在交易时段开放

## 扩展功能

可以基于此示例添加：
- 交易 API 实现 (tdapi.rs)
- 更多合约订阅
- 数据存储功能
- WebSocket 推送
- 策略交易逻辑