@echo off
setlocal enabledelayedexpansion

echo === CTP 交易组件环境设置 ===

:: 创建必要的目录
echo 创建目录结构...
if not exist "lib\windows" mkdir "lib\windows"
if not exist "ctp_flow" mkdir "ctp_flow"
if not exist "logs" mkdir "logs"

echo 目录结构创建完成

:: 检查 CTP 库文件
echo 检查 CTP 库文件...

set MISSING_LIBS=
if not exist "lib\windows\thostmduserapi_se.dll" (
    set MISSING_LIBS=!MISSING_LIBS! thostmduserapi_se.dll
)
if not exist "lib\windows\thosttraderapi_se.dll" (
    set MISSING_LIBS=!MISSING_LIBS! thosttraderapi_se.dll
)

if "!MISSING_LIBS!"=="" (
    echo ✅ 所有 CTP 库文件已就绪
) else (
    echo ❌ 缺少以下 CTP 库文件:
    for %%i in (!MISSING_LIBS!) do echo   - lib\windows\%%i
    echo.
    echo 请从以下地址下载 CTP API:
    echo   官网: http://www.sfit.com.cn/
    echo   下载页面: http://www.sfit.com.cn/DocumentDown.htm
    echo.
    echo 下载后请将库文件复制到 lib\windows\ 目录中
)

:: 检查配置文件
echo 检查配置文件...
if not exist "ctp_config.toml" (
    echo ❌ 配置文件 ctp_config.toml 不存在
    echo 应用程序首次运行时会自动创建默认配置文件
) else (
    echo ✅ 配置文件已存在
)

:: 检查 Rust 环境
echo 检查 Rust 环境...
rustc --version >nul 2>&1
if !errorlevel! equ 0 (
    for /f "tokens=*" %%i in ('rustc --version') do echo ✅ Rust 已安装: %%i
) else (
    echo ❌ 未检测到 Rust 环境
    echo 请访问 https://rustup.rs/ 安装 Rust
    exit /b 1
)

:: 检查 Bun 环境
echo 检查 Bun 环境...
bun --version >nul 2>&1
if !errorlevel! equ 0 (
    for /f "tokens=*" %%i in ('bun --version') do echo ✅ Bun 已安装: v%%i
) else (
    echo ❌ 未检测到 Bun 环境
    echo 请访问 https://bun.sh/ 安装 Bun
    exit /b 1
)

:: 检查 Tauri CLI
echo 检查 Tauri CLI...
tauri --version >nul 2>&1
if !errorlevel! equ 0 (
    for /f "tokens=*" %%i in ('tauri --version') do echo ✅ Tauri CLI 已安装: %%i
) else (
    echo ❌ 未检测到 Tauri CLI
    echo 正在安装 Tauri CLI...
    cargo install tauri-cli
    if !errorlevel! equ 0 (
        echo ✅ Tauri CLI 安装完成
    ) else (
        echo ❌ Tauri CLI 安装失败
        exit /b 1
    )
)

:: 环境变量配置示例
echo.
echo === 环境变量配置示例 ===
echo 可以通过以下环境变量覆盖配置文件设置:
echo.
echo set CTP_LIB_PATH=%cd%\lib\windows
echo set CTP_BROKER_ID=9999
echo set CTP_INVESTOR_ID=your_investor_id
echo set CTP_PASSWORD=your_password
echo set CTP_MD_FRONT_ADDR=tcp://180.168.146.187:10131
echo set CTP_TRADER_FRONT_ADDR=tcp://180.168.146.187:10130
echo.

:: 构建项目
echo === 构建项目 ===
echo 正在安装前端依赖...
bun install

echo 正在构建 Rust 后端...
cargo check

echo.
echo === 设置完成 ===
if "!MISSING_LIBS!"=="" (
    echo ✅ 环境设置完成，可以开始开发
    echo.
    echo 运行开发服务器:
    echo   bun run tauri:dev
    echo.
    echo 构建生产版本:
    echo   bun run tauri:build
) else (
    echo ⚠️  环境设置基本完成，但仍需要安装 CTP 库文件
    echo 请按照上述提示下载并安装 CTP 库文件后再运行应用程序
)

echo.
pause