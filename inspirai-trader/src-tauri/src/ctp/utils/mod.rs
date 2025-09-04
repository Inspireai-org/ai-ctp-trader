// 工具模块
// 包含数据转换、编码处理等辅助功能

pub mod converter;
pub mod encoding;

pub use converter::DataConverter;
pub use encoding::{gb18030_to_utf8, utf8_to_gb18030};