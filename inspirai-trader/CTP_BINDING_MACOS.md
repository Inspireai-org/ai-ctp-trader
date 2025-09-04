# CTP API macOS Framework 绑定说明

## 确认结果
✅ **macOS 下的 CTP 6.7.7 评测版本可以成功绑定到 Rust**

## 库信息
- **版本**: TraderapiMduserapi_6.7.7_CP_MacOS
- **框架格式**: macOS Framework (.framework)
- **支持架构**: x86_64 和 arm64（通用二进制）
- **包含框架**:
  - `thostmduserapi_se.framework` - 行情 API
  - `thosttraderapi_se.framework` - 交易 API

## 绑定方案

### 1. 构建配置 (build.rs)
- ✅ 自动检测并配置 Framework 路径
- ✅ 使用 `-F` 标志指定 Framework 搜索路径
- ✅ 使用 `framework=` 链接 Framework
- ✅ 设置运行时库搜索路径 (rpath)
- ✅ 支持 bindgen 自动生成 C++ 绑定

### 2. FFI 绑定层 (src/ctp/ffi.rs)
- ✅ Framework 可用性检查
- ✅ FFI 绑定管理器
- ✅ 字符串转换工具
- ✅ 资源生命周期管理

### 3. 系统绑定 (src/ctp/ctp_sys.rs)
- ✅ 自动生成绑定（通过 bindgen）
- ✅ 手动备用绑定（当 bindgen 不可用时）
- ✅ 基本数据结构定义

## 关键配置

### Cargo.toml 依赖
```toml
[build-dependencies]
bindgen = "0.70"  # 生成 C++ 绑定
cc = "1.0"        # 编译 C++ 代码

[dependencies]
libc = "0.2"      # FFI 操作
libloading = "0.8" # 动态库加载
```

### 框架符号链接修复
框架需要正确的符号链接结构：
```
framework/
├── Headers -> Versions/Current/Headers
├── Resources -> Versions/Current/Resources
├── thostmduserapi_se -> Versions/Current/thostmduserapi_se
└── Versions/
    ├── A/
    │   ├── Headers/
    │   ├── Resources/
    │   └── thostmduserapi_se (实际二进制)
    └── Current -> A
```

## 测试结果
所有测试项均通过：
- ✅ CTP 库文件可用性检查
- ✅ FFI 绑定实例创建
- ✅ 行情 API 创建（模拟）
- ✅ 交易 API 创建（模拟）
- ✅ 字符串转换（Rust ↔ C）
- ✅ 数据结构内存布局

## 下一步工作

### 1. C++ 桥接层
需要创建 C++ 包装器来：
- 实现 CTP SPI 回调接口
- 将 C++ 虚函数表转换为 C 函数指针
- 处理 C++ 异常

### 2. 异步接口
- 集成 tokio 异步运行时
- 实现事件驱动的回调处理
- 提供 async/await API

### 3. 安全封装
- 实现 Send + Sync 的线程安全包装
- 添加错误处理和恢复机制
- 提供高级 Rust API

## 注意事项

1. **代码签名**: macOS 可能需要对框架进行代码签名
2. **沙盒限制**: Tauri 应用可能需要额外的权限配置
3. **部署打包**: 需要将框架包含在最终的应用包中
4. **版本兼容**: 不同版本的 CTP API 可能不兼容

## 示例代码
运行测试：
```bash
cd src-tauri
cargo run --example test_ctp_binding
```

## 总结
macOS 下的 CTP 6.7.7 评测版本库可以成功绑定到 Rust。框架格式正确，支持 Intel 和 Apple Silicon 架构，构建系统已正确配置。下一步需要实现 C++ 桥接层来调用实际的 CTP API 功能。