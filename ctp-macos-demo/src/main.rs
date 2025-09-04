use clap::{Parser, ValueEnum};
use std::path::PathBuf;

mod mdapi;
use mdapi::*;

#[derive(Debug, Clone, ValueEnum)]
pub enum Environment {
    /// 模拟环境 (SimNow)
    Sim,
    /// 7x24小时测试环境 (TTS)
    Tts,
}

#[derive(Debug, Clone)]
pub struct CtpConfig {
    // 通用配置
    pub broker_id: String,
    pub user_id: String,
    pub password: String,
    pub app_id: String,
    pub auth_code: String,

    // MD API 配置
    pub md_front_address: String,
    pub md_dynlib_path: PathBuf,

    // TD API 配置 (预留)
    pub td_front_address: String,
    pub td_dynlib_path: PathBuf,
}

#[derive(Parser)]
#[command(author, version, about = "CTP macOS Demo - 基于 ctp2rs 的 CTP 接口示例", long_about = None)]
struct Args {
    /// 运行环境
    #[arg(short, long, value_enum, default_value_t = Environment::Sim)]
    environment: Environment,

    /// 用户ID
    #[arg(short, long, env = "CTP_USER_ID")]
    user_id: Option<String>,

    /// 密码
    #[arg(short, long, env = "CTP_PASSWORD")]
    password: Option<String>,

    /// 经纪商ID
    #[arg(short, long, env = "CTP_BROKER_ID")]
    broker_id: Option<String>,
}

fn create_config(env: Environment, args: Args) -> CtpConfig {
    // 使用提供的动态库路径
    #[cfg(target_os = "macos")]
    let md_dynlib_path = PathBuf::from("../inspirai-trader/lib/macos/TraderapiMduserapi_6.7.7_CP_MacOS/thostmduserapi_se.framework/thostmduserapi_se");
    
    #[cfg(target_os = "macos")]
    let td_dynlib_path = PathBuf::from("../inspirai-trader/lib/macos/TraderapiMduserapi_6.7.7_CP_MacOS/thosttraderapi_se.framework/thosttraderapi_se");

    #[cfg(not(target_os = "macos"))]
    compile_error!("此示例仅支持 macOS 平台");

    match env {
        Environment::Sim => {
            // SimNow 模拟环境配置
            CtpConfig {
                broker_id: args.broker_id.unwrap_or_else(|| "5071".to_string()),
                user_id: args.user_id.unwrap_or_else(|| {
                    println!("请提供用户ID (通过 -u 参数或 CTP_USER_ID 环境变量)");
                    std::process::exit(1);
                }),
                password: args.password.unwrap_or_else(|| {
                    println!("请提供密码 (通过 -p 参数或 CTP_PASSWORD 环境变量)");
                    std::process::exit(1);
                }),
                app_id: "inspirai_strategy_1.0.0".to_string(),
                auth_code: "0000000000000000".to_string(),
                md_front_address: "tcp://58.62.16.148:41214".to_string(), // SimNow 第一套
                td_front_address: "tcp://58.62.16.148:41206".to_string(),
                md_dynlib_path,
                td_dynlib_path,
            }
        }
        Environment::Tts => {
            // OpenCTP TTS 7x24 环境配置
            CtpConfig {
                broker_id: args.broker_id.unwrap_or_else(|| "9999".to_string()),
                user_id: args.user_id.unwrap_or_else(|| {
                    // TTS 环境的默认测试账号
                    println!("使用 TTS 测试账号，您也可以通过 -u 参数指定自己的账号");
                    "209992".to_string()
                }),
                password: args.password.unwrap_or_else(|| {
                    println!("使用 TTS 测试密码，您也可以通过 -p 参数指定自己的密码");
                    "CEE196Aa".to_string()
                }),
                app_id: "simnow_client_test".to_string(),
                auth_code: "0000000000000000".to_string(),
                md_front_address: "tcp://121.37.80.177:20004".to_string(), // OpenCTP TTS
                td_front_address: "tcp://121.37.80.177:20002".to_string(),
                md_dynlib_path,
                td_dynlib_path,
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    let env = args.environment.clone();
    
    println!("\n========================================");
    println!("     CTP macOS Demo - 基于 ctp2rs");
    println!("========================================");
    println!("运行环境: {:?}", env);
    
    let config = create_config(env, args);
    
    println!("经纪商ID: {}", config.broker_id);
    println!("用户ID: {}", config.user_id);
    println!("行情前置: {}", config.md_front_address);
    println!("----------------------------------------\n");

    // 仅运行行情接口
    run_md(config);
}
