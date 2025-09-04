# CTP 集成标准规范

## 核心原则

### 1. 禁止使用模拟实现
- **严禁**在生产代码中使用任何模拟的 CTP 功能
- **必须**使用真实的 ctp2rs 库进行 CTP API 集成
- **禁止**创建假的 CTP 响应数据或模拟回调
- **必须**确保所有 CTP 相关功能都通过真实的 API 调用实现

### 2. 强制使用 ctp2rs 库
- **必须**使用 `ctp2rs = { version = "0.1.7", features = ["ctp_v6_7_9"] }` 作为唯一的 CTP 接口
- **禁止**直接调用 C++ CTP API 或创建自定义 FFI 绑定
- **必须**使用 ctp2rs 提供的所有数据结构和转换工具
- **禁止**重新实现已有的 ctp2rs 功能

### 3. 真实 SPI 实现要求
```rust
// 正确的做法：实现真实的 ctp2rs trait
use ctp2rs::v1alpha1::{MdSpi, TraderSpi};

impl MdSpi for MdSpiImpl {
    fn on_front_connected(&mut self) {
        // 真实的连接处理逻辑
    }
    
    fn on_rtn_depth_market_data(&mut self, data: Option<&CThostFtdcDepthMarketDataField>) {
        // 真实的行情数据处理
    }
}

// 错误的做法：创建模拟的回调处理
// ❌ 禁止这样做
impl MdSpiImpl {
    pub fn simulate_market_data(&self) {
        // 这是错误的模拟实现
    }
}
```

### 4. 数据转换标准
- **必须**使用 ctp2rs 提供的数据转换工具
- **禁止**手动实现 GB18030 到 UTF-8 的转换
- **必须**使用 `ctp2rs::ffi::gb18030_cstr_i8_to_str` 等官方工具
- **禁止**创建自定义的 CTP 数据结构定义

```rust
// 正确的做法
use ctp2rs::ffi::gb18030_cstr_i8_to_str;
use ctp2rs::v1alpha1::CThostFtdcDepthMarketDataField;

fn convert_market_data(data: &CThostFtdcDepthMarketDataField) -> MarketDataTick {
    MarketDataTick {
        instrument_id: gb18030_cstr_i8_to_str(&data.InstrumentID).unwrap_or_default(),
        last_price: data.LastPrice,
        // ... 使用真实的 CTP 字段
    }
}

// 错误的做法：自定义结构体
// ❌ 禁止这样做
#[repr(C)]
struct CustomCtpMarketData {
    // 自定义字段定义
}
```

### 5. API 调用规范
- **必须**通过 ctp2rs 的 MdApi 和 TraderApi 进行所有 CTP 操作
- **禁止**绕过 ctp2rs 直接调用底层 API
- **必须**正确处理所有 CTP 错误码和响应
- **禁止**忽略或模拟 CTP 的错误处理

```rust
// 正确的做法
use ctp2rs::v1alpha1::{MdApi, TraderApi};

impl CtpClient {
    pub async fn subscribe_market_data(&mut self, instruments: &[String]) -> Result<(), CtpError> {
        if let Some(md_api) = &self.md_api {
            // 使用真实的 API 调用
            md_api.subscribe_market_data(instruments)?;
        }
        Ok(())
    }
}

// 错误的做法：模拟 API 调用
// ❌ 禁止这样做
impl CtpClient {
    pub async fn fake_subscribe(&self) {
        // 模拟订阅成功
        self.send_fake_success_event();
    }
}
```

### 6. 配置和环境管理
- **必须**使用真实的 CTP 服务器地址（SimNow、TTS、生产环境）
- **禁止**使用本地模拟服务器或假的地址
- **必须**正确配置 CTP 动态库路径
- **必须**验证所有 CTP 配置参数的有效性

```rust
// 正确的配置
impl CtpConfig {
    pub fn simnow_config() -> Self {
        Self {
            md_front_addr: "tcp://180.168.146.187:10131".to_string(), // 真实的 SimNow 地址
            trader_front_addr: "tcp://180.168.146.187:10130".to_string(),
            // ... 其他真实配置
        }
    }
}

// 错误的配置
// ❌ 禁止使用假地址
impl CtpConfig {
    pub fn fake_config() -> Self {
        Self {
            md_front_addr: "tcp://127.0.0.1:12345".to_string(), // 假地址
            // ...
        }
    }
}
```

### 7. 测试规范
- **允许**在单元测试中使用模拟数据，但必须明确标注
- **必须**提供集成测试使用真实的 CTP 环境
- **禁止**在生产代码路径中包含测试用的模拟逻辑
- **必须**将测试代码与生产代码严格分离

```rust
#[cfg(test)]
mod tests {
    // 测试中可以使用模拟数据
    fn create_mock_market_data() -> MarketDataTick {
        // 这是允许的，因为在测试代码中
    }
}

// 生产代码中禁止模拟
impl MarketDataService {
    // ❌ 禁止在生产代码中包含模拟逻辑
    #[cfg(feature = "mock")] // 即使有 feature flag 也不允许
    pub fn simulate_data(&self) {
        // 禁止的模拟实现
    }
}
```

### 8. 错误处理要求
- **必须**正确处理所有 CTP 错误码
- **必须**将 CTP 错误信息转换为中文用户友好的消息
- **禁止**忽略或掩盖 CTP 的真实错误
- **必须**记录所有 CTP 相关的错误和异常

```rust
impl CtpError {
    pub fn from_ctp_error(error_id: i32, error_msg: &str) -> Self {
        match error_id {
            0 => return Ok(()), // 成功不是错误
            -1 => CtpError::NetworkError("网络连接失败".to_string()),
            -2 => CtpError::AuthenticationError("用户名或密码错误".to_string()),
            -3 => CtpError::AuthenticationError("用户已登录".to_string()),
            // ... 处理所有已知错误码
            _ => CtpError::CtpApiError {
                code: error_id,
                message: error_msg.to_string(),
            },
        }
    }
}
```

### 9. 代码审查检查点
在代码审查时，必须检查以下项目：
- [ ] 是否使用了真实的 ctp2rs API 调用
- [ ] 是否存在任何模拟或假的 CTP 实现
- [ ] 是否正确处理了所有 CTP 回调
- [ ] 是否使用了真实的服务器地址和配置
- [ ] 是否正确实现了错误处理
- [ ] 是否遵循了 ctp2rs 的最佳实践

### 10. 部署前验证
在部署到任何环境前，必须验证：
- [ ] 所有 CTP 功能都使用真实的 API
- [ ] 没有任何模拟或测试代码在生产路径中
- [ ] CTP 动态库正确加载和链接
- [ ] 所有配置参数都是有效的
- [ ] 错误处理覆盖了所有可能的 CTP 错误场景

## 违规处理
如果发现违反以上规范的代码：
1. **立即停止**相关功能的开发和部署
2. **必须重构**违规代码，使用正确的 CTP 集成方式
3. **重新测试**所有相关功能
4. **更新文档**和测试用例

## 参考资源
- [ctp2rs 官方文档](https://docs.rs/ctp2rs/)
- [CTP API 官方文档](http://www.sfit.com.cn/5_2_DocumentDown.htm)
- 项目中的 `lib/README.md` - CTP 库配置说明
- 项目中的 `src-tauri/src/ctp/README.md` - CTP 模块使用指南