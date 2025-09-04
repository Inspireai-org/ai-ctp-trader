#!/bin/bash

echo "=== CTP macOS Demo 测试脚本 ==="
echo ""

# 构建项目
echo "1. 构建项目..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "构建失败"
    exit 1
fi
echo "✅ 构建成功"
echo ""

# 显示帮助信息
echo "2. 显示帮助信息..."
./target/release/ctp-macos-demo --help
echo ""

# 测试 TTS 环境（超时 3 秒）
echo "3. 测试 TTS 环境连接（3秒超时）..."
timeout 3 ./target/release/ctp-macos-demo 2>&1 | head -20
echo ""

# 检查流文件目录
echo "4. 检查流文件目录..."
if [ -d "ctp_md_flow" ]; then
    echo "✅ 流文件目录已创建: ctp_md_flow/"
    ls -la ctp_md_flow/ 2>/dev/null | head -5
else
    echo "❌ 流文件目录未创建"
fi
echo ""

echo "=== 测试完成 ==="
echo ""
echo "注意事项："
echo "- 错误码 4097 表示网络连接问题，可能是服务器地址变更或网络限制"
echo "- SimNow 仅在交易时段开放"
echo "- 可以尝试使用您自己的 SimNow 账号进行测试"
echo ""
echo "使用示例："
echo "  ./target/release/ctp-macos-demo -e sim -u YOUR_USER -p YOUR_PASS"