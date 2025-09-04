#!/bin/bash

# CTP 环境设置脚本
# 用于自动化配置 CTP 开发环境

set -e

echo "=== CTP 交易组件环境设置 ==="

# 检测操作系统
OS="unknown"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    OS="windows"
fi

echo "检测到操作系统: $OS"

# 创建必要的目录
echo "创建目录结构..."
mkdir -p lib/$OS
mkdir -p ctp_flow
mkdir -p logs

# 设置权限
chmod 755 lib/$OS
chmod 755 ctp_flow
chmod 755 logs

echo "目录结构创建完成"

# 检查 CTP 库文件
echo "检查 CTP 库文件..."

case $OS in
    "linux")
        REQUIRED_LIBS=("libthostmduserapi_se.so" "libthosttraderapi_se.so")
        ;;
    "macos")
        REQUIRED_LIBS=("libthostmduserapi_se.dylib" "libthosttraderapi_se.dylib")
        ;;
    "windows")
        REQUIRED_LIBS=("thostmduserapi_se.dll" "thosttraderapi_se.dll")
        ;;
    *)
        echo "不支持的操作系统: $OS"
        exit 1
        ;;
esac

MISSING_LIBS=()
for lib in "${REQUIRED_LIBS[@]}"; do
    if [ ! -f "lib/$OS/$lib" ]; then
        MISSING_LIBS+=("$lib")
    fi
done

if [ ${#MISSING_LIBS[@]} -eq 0 ]; then
    echo "✅ 所有 CTP 库文件已就绪"
else
    echo "❌ 缺少以下 CTP 库文件:"
    for lib in "${MISSING_LIBS[@]}"; do
        echo "  - lib/$OS/$lib"
    done
    echo ""
    echo "请从以下地址下载 CTP API:"
    echo "  官网: http://www.sfit.com.cn/"
    echo "  下载页面: http://www.sfit.com.cn/DocumentDown.htm"
    echo ""
    echo "下载后请将库文件复制到 lib/$OS/ 目录中"
fi

# 检查配置文件
echo "检查配置文件..."
if [ ! -f "ctp_config.toml" ]; then
    echo "❌ 配置文件 ctp_config.toml 不存在"
    echo "应用程序首次运行时会自动创建默认配置文件"
else
    echo "✅ 配置文件已存在"
fi

# 检查 Rust 环境
echo "检查 Rust 环境..."
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo "✅ Rust 已安装: $RUST_VERSION"
else
    echo "❌ 未检测到 Rust 环境"
    echo "请访问 https://rustup.rs/ 安装 Rust"
    exit 1
fi

# 检查 Bun 环境
echo "检查 Bun 环境..."
if command -v bun &> /dev/null; then
    BUN_VERSION=$(bun --version)
    echo "✅ Bun 已安装: v$BUN_VERSION"
else
    echo "❌ 未检测到 Bun 环境"
    echo "请访问 https://bun.sh/ 安装 Bun"
    exit 1
fi

# 检查 Tauri CLI
echo "检查 Tauri CLI..."
if command -v tauri &> /dev/null; then
    TAURI_VERSION=$(tauri --version)
    echo "✅ Tauri CLI 已安装: $TAURI_VERSION"
else
    echo "❌ 未检测到 Tauri CLI"
    echo "正在安装 Tauri CLI..."
    if command -v cargo &> /dev/null; then
        cargo install tauri-cli
        echo "✅ Tauri CLI 安装完成"
    else
        echo "❌ 无法安装 Tauri CLI，请先安装 Rust"
        exit 1
    fi
fi

# 设置环境变量示例
echo ""
echo "=== 环境变量配置示例 ==="
echo "可以通过以下环境变量覆盖配置文件设置:"
echo ""
echo "export CTP_LIB_PATH=\"$(pwd)/lib/$OS\""
echo "export CTP_BROKER_ID=\"9999\""
echo "export CTP_INVESTOR_ID=\"your_investor_id\""
echo "export CTP_PASSWORD=\"your_password\""
echo "export CTP_MD_FRONT_ADDR=\"tcp://180.168.146.187:10131\""
echo "export CTP_TRADER_FRONT_ADDR=\"tcp://180.168.146.187:10130\""
echo ""

# 构建项目
echo "=== 构建项目 ==="
echo "正在安装前端依赖..."
bun install

echo "正在构建 Rust 后端..."
cargo check

echo ""
echo "=== 设置完成 ==="
if [ ${#MISSING_LIBS[@]} -eq 0 ]; then
    echo "✅ 环境设置完成，可以开始开发"
    echo ""
    echo "运行开发服务器:"
    echo "  bun run tauri:dev"
    echo ""
    echo "构建生产版本:"
    echo "  bun run tauri:build"
else
    echo "⚠️  环境设置基本完成，但仍需要安装 CTP 库文件"
    echo "请按照上述提示下载并安装 CTP 库文件后再运行应用程序"
fi

echo ""