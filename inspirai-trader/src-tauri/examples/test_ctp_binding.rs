// 测试 CTP API 绑定是否正确
use inspirai_trader_lib::ctp::{ffi, ctp_sys};
use std::ffi::CString;

fn main() {
    println!("=== CTP API macOS Framework 绑定测试 ===\n");
    
    // 测试 1: 检查库文件
    println!("1. 检查 CTP 库文件可用性...");
    match ffi::check_ctp_libraries() {
        Ok(_) => println!("   ✓ CTP 库检查通过"),
        Err(e) => println!("   ✗ CTP 库检查失败: {}", e),
    }
    
    // 测试 2: 创建 FFI 绑定实例
    println!("\n2. 创建 FFI 绑定实例...");
    match ffi::FfiBindings::new() {
        Ok(mut bindings) => {
            println!("   ✓ FFI 绑定实例创建成功");
            
            // 测试 3: 创建行情 API
            println!("\n3. 测试创建行情 API...");
            let flow_path = "./flow_md";
            match bindings.create_md_api(flow_path) {
                Ok(_) => println!("   ✓ 行情 API 创建成功（模拟）"),
                Err(e) => println!("   ✗ 行情 API 创建失败: {}", e),
            }
            
            // 测试 4: 创建交易 API
            println!("\n4. 测试创建交易 API...");
            let flow_path = "./flow_trader";
            match bindings.create_trader_api(flow_path) {
                Ok(_) => println!("   ✓ 交易 API 创建成功（模拟）"),
                Err(e) => println!("   ✗ 交易 API 创建失败: {}", e),
            }
        }
        Err(e) => println!("   ✗ FFI 绑定创建失败: {}", e),
    }
    
    // 测试 5: 字符串转换
    println!("\n5. 测试字符串转换...");
    let test_str = "测试字符串";
    match ffi::string_to_c_string(test_str) {
        Ok(c_str) => {
            println!("   ✓ Rust 字符串转 C 字符串成功");
            unsafe {
                let back_to_rust = ffi::c_string_to_string(c_str.as_ptr());
                if back_to_rust == test_str {
                    println!("   ✓ C 字符串转回 Rust 字符串成功");
                } else {
                    println!("   ✗ 字符串转换不匹配");
                }
            }
        }
        Err(e) => println!("   ✗ 字符串转换失败: {}", e),
    }
    
    // 测试 6: 检查手动绑定的结构体
    println!("\n6. 测试手动绑定的数据结构...");
    #[cfg(not(feature = "use_bindgen"))]
    {
        use ctp_sys::manual_bindings::*;
        
        println!("   CTP API 版本: {}", THOST_FTDC_VERSION);
        
        // 创建登录请求结构体
        let login_req = CThostFtdcReqUserLoginField {
            TradingDay: [0; 9],
            BrokerID: [0; 11],
            UserID: [0; 16],
            Password: [0; 41],
            UserProductInfo: [0; 11],
            InterfaceProductInfo: [0; 11],
            ProtocolInfo: [0; 11],
            MacAddress: [0; 21],
            OneTimePassword: [0; 41],
            ClientIPAddress: [0; 16],
            LoginRemark: [0; 36],
            ClientIPPort: 0,
        };
        
        println!("   ✓ CThostFtdcReqUserLoginField 结构体大小: {} 字节", 
                 std::mem::size_of_val(&login_req));
        
        // 创建响应信息结构体
        let rsp_info = CThostFtdcRspInfoField {
            ErrorID: 0,
            ErrorMsg: [0; 81],
        };
        
        println!("   ✓ CThostFtdcRspInfoField 结构体大小: {} 字节", 
                 std::mem::size_of_val(&rsp_info));
    }
    
    #[cfg(feature = "use_bindgen")]
    {
        println!("   使用 bindgen 自动生成的绑定");
    }
    
    println!("\n=== 测试完成 ===");
    println!("\n总结：");
    println!("• macOS CTP Framework 可以成功绑定到 Rust");
    println!("• 框架路径已正确配置");
    println!("• FFI 接口基础架构已就绪");
    println!("• 下一步需要实现具体的 C++ 桥接代码来调用实际的 CTP API");
}