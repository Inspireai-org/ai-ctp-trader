use std::env;
use std::path::PathBuf;

fn main() {
    // Tauri 构建脚本
    tauri_build::build();
    
    // CTP 库配置
    setup_ctp_libraries();
    
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CTP_LIB_PATH");
}

fn setup_ctp_libraries() {
    println!("cargo:rustc-cfg=feature=\"ctp\"");
    
    // 检查是否在测试模式下运行
    let is_test = env::var("CARGO_CFG_TEST").is_ok();
    
    // 如果在测试模式下且没有设置强制链接 CTP 库的环境变量，则跳过链接
    if is_test && env::var("FORCE_CTP_LINK").is_err() {
        println!("cargo:warning=跳过 CTP 库链接（测试模式）");
        return;
    }
    
    // 获取 CTP 库路径
    let ctp_lib_path = env::var("CTP_LIB_PATH")
        .unwrap_or_else(|_| {
            // 获取项目根目录的绝对路径
            let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            let base_path = PathBuf::from(manifest_dir)
                .parent()
                .unwrap()
                .to_path_buf();
            
            // 默认库路径
            if cfg!(target_os = "windows") {
                base_path.join("lib/windows").to_str().unwrap().to_string()
            } else if cfg!(target_os = "linux") {
                base_path.join("lib/linux").to_str().unwrap().to_string()
            } else if cfg!(target_os = "macos") {
                base_path.join("lib/macos/6.7.7/product").to_str().unwrap().to_string()
            } else {
                base_path.join("lib").to_str().unwrap().to_string()
            }
        });
    
    // 检查库路径是否存在
    if !std::path::Path::new(&ctp_lib_path).exists() {
        println!("cargo:warning=CTP 库路径不存在: {}", ctp_lib_path);
        println!("cargo:warning=请确保 CTP 库文件已正确放置");
        return;
    }
    
    // 根据平台链接不同的库
    if cfg!(target_os = "windows") {
        // Windows 平台的 CTP 库
        println!("cargo:rustc-link-search=native={}", ctp_lib_path);
        println!("cargo:rustc-link-lib=dylib=thostmduserapi_se");
        println!("cargo:rustc-link-lib=dylib=thosttraderapi_se");
    } else if cfg!(target_os = "linux") {
        // Linux 平台的 CTP 库
        println!("cargo:rustc-link-search=native={}", ctp_lib_path);
        println!("cargo:rustc-link-lib=dylib=thostmduserapi_se");
        println!("cargo:rustc-link-lib=dylib=thosttraderapi_se");
    } else if cfg!(target_os = "macos") {
        // macOS 平台的 CTP Framework
        // 对于 Framework，需要使用 -F 标志指定框架搜索路径
        println!("cargo:rustc-link-search=framework={}", ctp_lib_path);
        
        // 链接 Framework（不需要 .framework 后缀）
        println!("cargo:rustc-link-lib=framework=thostmduserapi_se");
        println!("cargo:rustc-link-lib=framework=thosttraderapi_se");
        
        // 设置运行时库搜索路径（rpath）
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/../Frameworks");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", ctp_lib_path);
    }
    
    // 生成 FFI 绑定（暂时禁用，使用 ctp2rs 库）
    // #[cfg(target_os = "macos")]
    // generate_macos_bindings(&ctp_lib_path);
}

// 为 macOS 生成 C++ 绑定（暂时禁用）
#[allow(dead_code)]
fn generate_macos_bindings(ctp_lib_path: &str) {
    // 设置头文件搜索路径
    let md_framework_path = format!("{}/thostmduserapi_se.framework/Versions/A/Headers", ctp_lib_path);
    let trader_framework_path = format!("{}/thosttraderapi_se.framework/Versions/A/Headers", ctp_lib_path);
    
    println!("cargo:warning=生成 CTP API 绑定...");
    println!("cargo:warning=行情 API 头文件路径: {}", md_framework_path);
    println!("cargo:warning=交易 API 头文件路径: {}", trader_framework_path);
    
    // 创建包装头文件内容
    let wrapper_content = format!(r#"
#ifndef WRAPPER_H
#define WRAPPER_H

// 行情 API
#include "ThostFtdcMdApi.h"

// 交易 API
#include "ThostFtdcTraderApi.h"

#endif // WRAPPER_H
    "#);
    
    // 写入临时包装头文件
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let wrapper_path = out_dir.join("wrapper.h");
    std::fs::write(&wrapper_path, wrapper_content).expect("Failed to write wrapper.h");
    
    // 使用 bindgen 生成绑定（暂时禁用，使用 ctp2rs 库）
    /*
    let bindings = bindgen::Builder::default()
        .header(wrapper_path.to_str().unwrap())
        .clang_arg(format!("-I{}", md_framework_path))
        .clang_arg(format!("-I{}", trader_framework_path))
        // 允许的函数和类型
        .allowlist_function(".*Ftdc.*")
        .allowlist_type(".*Ftdc.*")
        .allowlist_var(".*THOST.*")
        // C++ 特性
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++11")
        // 生成注释
        .generate_comments(true)
        // 处理回调
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate();
        
    match bindings {
        Ok(b) => {
            b.write_to_file(out_dir.join("ctp_bindings.rs"))
                .expect("Couldn't write bindings!");
            println!("cargo:warning=CTP 绑定生成成功");
        },
        Err(e) => {
            println!("cargo:warning=CTP 绑定生成失败: {}", e);
            println!("cargo:warning=将使用手动定义的 FFI 接口");
        }
    }
    */
    
    println!("cargo:warning=使用 ctp2rs 库，跳过自定义绑定生成");
}