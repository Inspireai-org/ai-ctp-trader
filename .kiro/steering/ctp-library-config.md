# CTP 库配置规范

## 库文件结构
项目中包含 CTP 6.7.7 版本的 macOS 库文件，位于 `lib/macos/6.7.7/` 目录下：

```
lib/macos/6.7.7/
├── cepin/                           # 测评环境库
│   ├── thostmduserapi_se.framework/ # 行情 API 框架
│   └── thosttraderapi_se.framework/ # 交易 API 框架
└── product/                         # 生产环境库
    ├── thostmduserapi_se.framework/ # 行情 API 框架
    └── thosttraderapi_se.framework/ # 交易 API 框架
```

## 库版本信息
- **CTP 版本**: 6.7.7
- **支持架构**: x86_64 和 arm64
- **系统要求**: macOS 10.15 以上版本
- **开发环境**: 需要配置 Xcode 开发环境

## 新增功能
本版本新增接口：
1. `GetFrontInfo` - 获取已连接的前置信息
2. `ReqQryInvestorPortfSetting` - 投资者新组保设置查询

## 环境区分
- **测评环境 (cepin)**: 用于开发和测试，连接测评服务器
- **生产环境 (product)**: 用于正式交易，连接生产服务器

## Rust FFI 集成配置
在 `Cargo.toml` 中配置库链接：

```toml
[target.'cfg(target_os = "macos")'.dependencies]
# CTP 库的 FFI 绑定将通过 build.rs 脚本动态配置

[build-dependencies]
bindgen = "0.69"
```

## 构建脚本配置
在 `build.rs` 中根据环境变量选择对应的库：

```rust
fn main() {
    let env = std::env::var("CTP_ENV").unwrap_or_else(|_| "cepin".to_string());
    let lib_path = match env.as_str() {
        "product" => "lib/macos/6.7.7/product",
        _ => "lib/macos/6.7.7/cepin", // 默认使用测评环境
    };
    
    println!("cargo:rustc-link-search=framework={}", lib_path);
    println!("cargo:rustc-link-lib=framework=thosttraderapi_se");
    println!("cargo:rustc-link-lib=framework=thostmduserapi_se");
}
```

## 头文件引用
在 FFI 绑定中使用正确的头文件路径：
```rust
// 交易 API
#include "thosttraderapi_se/ThostFtdcTraderApi.h"
// 行情 API  
#include "thostmduserapi_se/ThostFtdcMdApi.h"
```

## 权限配置
应用需要申请以下权限：
- 网络访问权限
- 文件系统访问权限（用于日志和配置文件）

## Tauri 配置
在 `tauri.conf.json` 中配置框架嵌入：
```json
{
  "bundle": {
    "macOS": {
      "frameworks": [
        "lib/macos/6.7.7/cepin/thosttraderapi_se.framework",
        "lib/macos/6.7.7/cepin/thostmduserapi_se.framework"
      ]
    }
  }
}
```

## 环境切换
通过环境变量 `CTP_ENV` 控制使用的库版本：
- `CTP_ENV=cepin` - 使用测评环境库（默认）
- `CTP_ENV=product` - 使用生产环境库

## 故障排除
1. **头文件找不到**: 确保引用路径为 `thosttraderapi_se/ThostFtdcTraderApi.h`
2. **程序运行失败**: 在 Xcode 中设置 SDK Embed 为 "Embed & Sign"
3. **架构不匹配**: 确保库支持当前系统架构（x86_64 或 arm64）

## 安全注意事项
- 生产环境库文件需要妥善保管，不应提交到公共代码仓库
- 测评环境和生产环境的配置要严格区分
- API 密钥和证书文件要加密存储