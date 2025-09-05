# CTP 动态库文件目录

此目录用于存放 CTP (Comprehensive Transaction Platform) 的动态库文件。

## 目录结构

```
lib/
├── windows/          # Windows 平台库文件
│   ├── thostmduserapi_se.dll
│   ├── thosttraderapi_se.dll
│   └── ...
├── linux/            # Linux 平台库文件
│   ├── libthostmduserapi_se.so
│   ├── libthosttraderapi_se.so
│   └── ...
├── macos/            # macOS 平台库文件
│   ├── libthostmduserapi_se.dylib
│   ├── libthosttraderapi_se.dylib
│   └── ...
└── README.md
```

## 安装说明

### 1. 获取 CTP 库文件

从上海期货信息技术有限公司官网下载对应平台的 CTP API：
- 官网地址: http://www.sfit.com.cn/
- 下载页面: http://www.sfit.com.cn/DocumentDown.htm

### 2. 安装库文件

#### Windows 平台
1. 下载 Windows 版本的 CTP API
2. 解压后将以下文件复制到 `lib/windows/` 目录：
   - `thostmduserapi_se.dll` (行情 API)
   - `thosttraderapi_se.dll` (交易 API)
   - 相关的依赖库文件

#### Linux 平台
1. 下载 Linux 版本的 CTP API
2. 解压后将以下文件复制到 `lib/linux/` 目录：
   - `libthostmduserapi_se.so` (行情 API)
   - `libthosttraderapi_se.so` (交易 API)
   - 相关的依赖库文件

#### macOS 平台
1. 下载 macOS 版本的 CTP API（如果可用）
2. 解压后将以下文件复制到 `lib/macos/` 目录：
   - `libthostmduserapi_se.dylib` (行情 API)
   - `libthosttraderapi_se.dylib` (交易 API)
   - 相关的依赖库文件

### 3. 环境变量配置

可以通过设置 `CTP_LIB_PATH` 环境变量来指定自定义的库文件路径：

```bash
export CTP_LIB_PATH=/path/to/your/ctp/libs
```

### 4. 验证安装

运行应用程序时，系统会自动检查库文件是否存在。如果缺少必要的库文件，会在日志中显示相应的错误信息。

## 注意事项

1. **版本兼容性**: 确保使用的 CTP API 版本与应用程序兼容
2. **平台匹配**: 库文件必须与目标平台架构匹配（x86/x64）
3. **依赖关系**: 某些 CTP 库可能依赖其他系统库，请确保系统已安装相关依赖
4. **许可协议**: 使用 CTP API 需要遵守相应的许可协议

## 故障排除

### 常见问题

1. **库文件未找到**
   - 检查文件路径是否正确
   - 确认文件权限是否允许读取
   - 验证环境变量设置

2. **库文件加载失败**
   - 检查库文件是否损坏
   - 确认平台架构匹配
   - 检查依赖库是否完整

3. **版本不兼容**
   - 更新到兼容的 CTP API 版本
   - 检查应用程序的版本要求

如有问题，请查看应用程序日志获取详细的错误信息。