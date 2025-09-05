use serde_json;
use super::{LogEntry, error::LogError, config::LogLevel};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

/// 日志格式化器 trait
pub trait LogFormatter: std::fmt::Debug + Send + Sync {
    /// 格式化日志条目为字符串
    fn format(&self, entry: &LogEntry) -> Result<String, LogError>;
    
    /// 获取格式化器名称
    fn name(&self) -> &'static str;
    
    /// 是否支持彩色输出
    fn supports_color(&self) -> bool {
        false
    }
    
    /// 获取格式化选项
    fn get_options(&self) -> FormatterOptions {
        FormatterOptions::default()
    }
}

/// 格式化器选项
#[derive(Debug, Clone)]
pub struct FormatterOptions {
    pub include_timestamp: bool,
    pub include_level: bool,
    pub include_module: bool,
    pub include_thread: bool,
    pub include_fields: bool,
    pub timestamp_format: String,
    pub use_colors: bool,
    pub max_message_length: Option<usize>,
    pub indent_size: usize,
}

impl Default for FormatterOptions {
    fn default() -> Self {
        Self {
            include_timestamp: true,
            include_level: true,
            include_module: true,
            include_thread: false,
            include_fields: true,
            timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
            use_colors: false,
            max_message_length: None,
            indent_size: 2,
        }
    }
}

/// JSON 格式化器 - 用于结构化日志
#[derive(Debug, Clone)]
pub struct JsonFormatter {
    options: FormatterOptions,
    pretty_print: bool,
}

impl JsonFormatter {
    /// 创建新的 JSON 格式化器
    pub fn new() -> Self {
        Self {
            options: FormatterOptions::default(),
            pretty_print: false,
        }
    }
    
    /// 创建美化输出的 JSON 格式化器
    pub fn pretty() -> Self {
        Self {
            options: FormatterOptions::default(),
            pretty_print: true,
        }
    }
    
    /// 设置格式化选项
    pub fn with_options(mut self, options: FormatterOptions) -> Self {
        self.options = options;
        self
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl LogFormatter for JsonFormatter {
    fn format(&self, entry: &LogEntry) -> Result<String, LogError> {
        let mut json_obj = serde_json::Map::new();
        
        // 时间戳
        if self.options.include_timestamp {
            json_obj.insert(
                "timestamp".to_string(),
                serde_json::Value::String(entry.timestamp.format(&self.options.timestamp_format).to_string())
            );
        }
        
        // 日志级别
        if self.options.include_level {
            json_obj.insert(
                "level".to_string(),
                serde_json::Value::String(entry.level.as_str().to_string())
            );
        }
        
        // 模块名
        if self.options.include_module {
            json_obj.insert(
                "module".to_string(),
                serde_json::Value::String(entry.module.clone())
            );
        }
        
        // 线程ID
        if self.options.include_thread {
            json_obj.insert(
                "thread".to_string(),
                serde_json::Value::String(entry.thread_id.clone())
            );
        }
        
        // 消息
        let message = if let Some(max_len) = self.options.max_message_length {
            if entry.message.len() > max_len {
                format!("{}...", &entry.message[..max_len])
            } else {
                entry.message.clone()
            }
        } else {
            entry.message.clone()
        };
        json_obj.insert("message".to_string(), serde_json::Value::String(message));
        
        // 请求ID
        if let Some(request_id) = &entry.request_id {
            json_obj.insert(
                "request_id".to_string(),
                serde_json::Value::String(request_id.clone())
            );
        }
        
        // 会话ID
        if let Some(session_id) = &entry.session_id {
            json_obj.insert(
                "session_id".to_string(),
                serde_json::Value::String(session_id.clone())
            );
        }
        
        // 额外字段
        if self.options.include_fields {
            for (key, value) in &entry.fields {
                json_obj.insert(key.clone(), value.clone());
            }
        }
        
        // 序列化为 JSON
        let json_value = serde_json::Value::Object(json_obj);
        let result = if self.pretty_print {
            serde_json::to_string_pretty(&json_value)
        } else {
            serde_json::to_string(&json_value)
        };
        
        match result {
            Ok(mut json_str) => {
                json_str.push('\n');
                Ok(json_str)
            }
            Err(e) => Err(LogError::SerializationError(e)),
        }
    }
    
    fn name(&self) -> &'static str {
        "json"
    }
    
    fn get_options(&self) -> FormatterOptions {
        self.options.clone()
    }
}

/// 人类可读格式化器 - 用于控制台和文本文件
#[derive(Debug, Clone)]
pub struct HumanReadableFormatter {
    options: FormatterOptions,
    field_separator: String,
    level_colors: HashMap<LogLevel, &'static str>,
    reset_color: &'static str,
}

impl HumanReadableFormatter {
    /// 创建新的人类可读格式化器
    pub fn new() -> Self {
        let mut level_colors = HashMap::new();
        level_colors.insert(LogLevel::Trace, "\x1b[37m"); // 白色
        level_colors.insert(LogLevel::Debug, "\x1b[36m"); // 青色
        level_colors.insert(LogLevel::Info, "\x1b[32m");  // 绿色
        level_colors.insert(LogLevel::Warn, "\x1b[33m");  // 黄色
        level_colors.insert(LogLevel::Error, "\x1b[31m"); // 红色
        
        Self {
            options: FormatterOptions::default(),
            field_separator: " ".to_string(),
            level_colors,
            reset_color: "\x1b[0m",
        }
    }
    
    /// 创建带颜色的格式化器
    pub fn with_colors() -> Self {
        let mut formatter = Self::new();
        formatter.options.use_colors = true;
        formatter
    }
    
    /// 设置字段分隔符
    pub fn with_separator(mut self, separator: &str) -> Self {
        self.field_separator = separator.to_string();
        self
    }
    
    /// 设置格式化选项
    pub fn with_options(mut self, options: FormatterOptions) -> Self {
        self.options = options;
        self
    }
    
    /// 格式化级别字符串
    fn format_level(&self, level: LogLevel) -> String {
        let level_str = format!("[{:5}]", level.as_str());
        
        if self.options.use_colors {
            if let Some(&color) = self.level_colors.get(&level) {
                format!("{}{}{}", color, level_str, self.reset_color)
            } else {
                level_str
            }
        } else {
            level_str
        }
    }
    
    /// 格式化字段
    fn format_fields(&self, fields: &HashMap<String, serde_json::Value>) -> String {
        if fields.is_empty() {
            return String::new();
        }
        
        let mut field_parts = Vec::new();
        
        // 按键名排序以确保输出一致
        let mut sorted_fields: Vec<_> = fields.iter().collect();
        sorted_fields.sort_by_key(|(k, _)| *k);
        
        for (key, value) in sorted_fields {
            let formatted_value = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => "null".to_string(),
                _ => value.to_string(),
            };
            
            field_parts.push(format!("{}={}", key, formatted_value));
        }
        
        field_parts.join(&self.field_separator)
    }
}

impl Default for HumanReadableFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl LogFormatter for HumanReadableFormatter {
    fn format(&self, entry: &LogEntry) -> Result<String, LogError> {
        let mut parts = Vec::new();
        
        // 时间戳
        if self.options.include_timestamp {
            parts.push(entry.timestamp.format(&self.options.timestamp_format).to_string());
        }
        
        // 日志级别
        if self.options.include_level {
            parts.push(self.format_level(entry.level));
        }
        
        // 模块名
        if self.options.include_module {
            parts.push(format!("[{}]", entry.module));
        }
        
        // 线程ID
        if self.options.include_thread {
            parts.push(format!("[{}]", entry.thread_id));
        }
        
        // 消息
        let message = if let Some(max_len) = self.options.max_message_length {
            if entry.message.len() > max_len {
                format!("{}...", &entry.message[..max_len])
            } else {
                entry.message.clone()
            }
        } else {
            entry.message.clone()
        };
        parts.push(message);
        
        // 请求ID和会话ID
        if let Some(request_id) = &entry.request_id {
            parts.push(format!("req_id={}", request_id));
        }
        
        if let Some(session_id) = &entry.session_id {
            parts.push(format!("sess_id={}", session_id));
        }
        
        // 额外字段
        if self.options.include_fields && !entry.fields.is_empty() {
            parts.push(self.format_fields(&entry.fields));
        }
        
        let result = parts.join(&self.field_separator);
        Ok(format!("{}\n", result))
    }
    
    fn name(&self) -> &'static str {
        "human_readable"
    }
    
    fn supports_color(&self) -> bool {
        true
    }
    
    fn get_options(&self) -> FormatterOptions {
        self.options.clone()
    }
}

/// 紧凑格式化器 - 用于高频日志场景
#[derive(Debug, Clone)]
pub struct CompactFormatter {
    options: FormatterOptions,
}

impl CompactFormatter {
    /// 创建新的紧凑格式化器
    pub fn new() -> Self {
        let mut options = FormatterOptions::default();
        options.include_thread = false;
        options.timestamp_format = "%H:%M:%S%.3f".to_string(); // 只包含时间
        
        Self { options }
    }
    
    /// 设置格式化选项
    pub fn with_options(mut self, options: FormatterOptions) -> Self {
        self.options = options;
        self
    }
}

impl Default for CompactFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl LogFormatter for CompactFormatter {
    fn format(&self, entry: &LogEntry) -> Result<String, LogError> {
        let mut result = String::new();
        
        // 时间戳（简短格式）
        if self.options.include_timestamp {
            result.push_str(&entry.timestamp.format(&self.options.timestamp_format).to_string());
            result.push(' ');
        }
        
        // 级别（单字符）
        if self.options.include_level {
            let level_char = match entry.level {
                LogLevel::Trace => 'T',
                LogLevel::Debug => 'D',
                LogLevel::Info => 'I',
                LogLevel::Warn => 'W',
                LogLevel::Error => 'E',
            };
            result.push(level_char);
            result.push(' ');
        }
        
        // 模块名（简化）
        if self.options.include_module {
            let module_short = entry.module
                .split("::")
                .last()
                .unwrap_or(&entry.module);
            result.push_str(&format!("[{}] ", module_short));
        }
        
        // 消息
        let message = if let Some(max_len) = self.options.max_message_length {
            if entry.message.len() > max_len {
                &entry.message[..max_len]
            } else {
                &entry.message
            }
        } else {
            &entry.message
        };
        result.push_str(message);
        
        // 关键字段（紧凑格式）
        if self.options.include_fields {
            let key_fields = ["request_id", "account_id", "instrument_id"];
            let mut field_parts = Vec::new();
            
            for field in &key_fields {
                if let Some(value) = entry.fields.get(*field) {
                    if let Some(str_value) = value.as_str() {
                        field_parts.push(format!("{}={}", field, str_value));
                    }
                }
            }
            
            if !field_parts.is_empty() {
                result.push_str(" [");
                result.push_str(&field_parts.join(" "));
                result.push(']');
            }
        }
        
        result.push('\n');
        Ok(result)
    }
    
    fn name(&self) -> &'static str {
        "compact"
    }
    
    fn get_options(&self) -> FormatterOptions {
        self.options.clone()
    }
}

/// CSV 格式化器 - 用于数据分析
#[derive(Debug)]
pub struct CsvFormatter {
    options: FormatterOptions,
    delimiter: char,
    include_header: bool,
    header_written: AtomicBool,
}

impl CsvFormatter {
    /// 创建新的 CSV 格式化器
    pub fn new() -> Self {
        Self {
            options: FormatterOptions::default(),
            delimiter: ',',
            include_header: true,
            header_written: AtomicBool::new(false),
        }
    }
    
    /// 设置分隔符
    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }
    
    /// 设置是否包含标题行
    pub fn with_header(mut self, include_header: bool) -> Self {
        self.include_header = include_header;
        self
    }
    
    /// 转义 CSV 字段
    fn escape_csv_field(&self, field: &str) -> String {
        if field.contains(self.delimiter) || field.contains('"') || field.contains('\n') {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }
    
    /// 获取 CSV 标题行
    fn get_header(&self) -> String {
        let mut headers = Vec::new();
        
        if self.options.include_timestamp {
            headers.push("timestamp");
        }
        if self.options.include_level {
            headers.push("level");
        }
        if self.options.include_module {
            headers.push("module");
        }
        if self.options.include_thread {
            headers.push("thread");
        }
        
        headers.extend_from_slice(&["message", "request_id", "session_id"]);
        
        format!("{}\n", headers.join(&self.delimiter.to_string()))
    }
}

impl Default for CsvFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl LogFormatter for CsvFormatter {
    fn format(&self, entry: &LogEntry) -> Result<String, LogError> {
        let mut result = String::new();
        
        // 写入标题行（仅一次）
        if self.include_header && !self.header_written.load(Ordering::Relaxed) {
            result.push_str(&self.get_header());
            self.header_written.store(true, Ordering::Relaxed);
        }
        
        let mut fields = Vec::new();
        
        // 时间戳
        if self.options.include_timestamp {
            fields.push(self.escape_csv_field(&entry.timestamp.format(&self.options.timestamp_format).to_string()));
        }
        
        // 日志级别
        if self.options.include_level {
            fields.push(entry.level.as_str().to_string());
        }
        
        // 模块名
        if self.options.include_module {
            fields.push(self.escape_csv_field(&entry.module));
        }
        
        // 线程ID
        if self.options.include_thread {
            fields.push(self.escape_csv_field(&entry.thread_id));
        }
        
        // 消息
        let message = if let Some(max_len) = self.options.max_message_length {
            if entry.message.len() > max_len {
                format!("{}...", &entry.message[..max_len])
            } else {
                entry.message.clone()
            }
        } else {
            entry.message.clone()
        };
        fields.push(self.escape_csv_field(&message));
        
        // 请求ID和会话ID
        fields.push(entry.request_id.as_deref().unwrap_or("").to_string());
        fields.push(entry.session_id.as_deref().unwrap_or("").to_string());
        
        result.push_str(&fields.join(&self.delimiter.to_string()));
        result.push('\n');
        
        Ok(result)
    }
    
    fn name(&self) -> &'static str {
        "csv"
    }
    
    fn get_options(&self) -> FormatterOptions {
        self.options.clone()
    }
}

/// 格式化器工厂
pub struct FormatterFactory;

impl FormatterFactory {
    /// 根据名称创建格式化器
    pub fn create(name: &str) -> Result<Box<dyn LogFormatter + Send>, LogError> {
        match name.to_lowercase().as_str() {
            "json" => Ok(Box::new(JsonFormatter::new())),
            "json_pretty" => Ok(Box::new(JsonFormatter::pretty())),
            "human" | "human_readable" => Ok(Box::new(HumanReadableFormatter::new())),
            "human_color" => Ok(Box::new(HumanReadableFormatter::with_colors())),
            "compact" => Ok(Box::new(CompactFormatter::new())),
            "csv" => Ok(Box::new(CsvFormatter::new())),
            _ => Err(LogError::InvalidConfig {
                field: format!("不支持的格式化器: {}", name),
            }),
        }
    }
    
    /// 获取所有支持的格式化器名称
    pub fn supported_formatters() -> Vec<&'static str> {
        vec![
            "json",
            "json_pretty",
            "human",
            "human_readable",
            "human_color",
            "compact",
            "csv",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::context::LogContext;
    use std::collections::HashMap;

    fn create_test_entry() -> LogEntry {
        let mut context = LogContext::new(LogLevel::Info, "test_module");
        context.request_id = Some("req_123".to_string());
        context.session_id = Some("sess_456".to_string());
        
        let mut fields = HashMap::new();
        fields.insert("account_id".to_string(), serde_json::Value::String("12345".to_string()));
        fields.insert("instrument_id".to_string(), serde_json::Value::String("rb2405".to_string()));
        fields.insert("price".to_string(), serde_json::Value::Number(3850.5.into()));
        
        LogEntry {
            timestamp: chrono::Utc::now(),
            level: LogLevel::Info,
            module: "test_module".to_string(),
            thread_id: "test_thread".to_string(),
            message: "Test log message".to_string(),
            context,
            request_id: Some("req_123".to_string()),
            session_id: Some("sess_456".to_string()),
            fields,
        }
    }
    
    #[test]
    fn test_json_formatter() {
        let formatter = JsonFormatter::new();
        let entry = create_test_entry();
        
        let result = formatter.format(&entry);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        assert!(formatted.contains("\"level\":\"INFO\""));
        assert!(formatted.contains("\"module\":\"test_module\""));
        assert!(formatted.contains("\"message\":\"Test log message\""));
        assert!(formatted.contains("\"request_id\":\"req_123\""));
        assert!(formatted.contains("\"account_id\":\"12345\""));
        
        // 验证是否为有效 JSON
        let parsed: serde_json::Value = serde_json::from_str(formatted.trim()).unwrap();
        assert!(parsed.is_object());
    }
    
    #[test]
    fn test_json_pretty_formatter() {
        let formatter = JsonFormatter::pretty();
        let entry = create_test_entry();
        
        let result = formatter.format(&entry);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        // 美化格式应该包含换行和缩进
        assert!(formatted.contains("{\n"));
        assert!(formatted.contains("  \"level\""));
    }
    
    #[test]
    fn test_human_readable_formatter() {
        let formatter = HumanReadableFormatter::new();
        let entry = create_test_entry();
        
        let result = formatter.format(&entry);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        assert!(formatted.contains("[INFO ]"));
        assert!(formatted.contains("[test_module]"));
        assert!(formatted.contains("Test log message"));
        assert!(formatted.contains("req_id=req_123"));
        assert!(formatted.contains("account_id=12345"));
    }
    
    #[test]
    fn test_human_readable_with_colors() {
        let formatter = HumanReadableFormatter::with_colors();
        let entry = create_test_entry();
        
        let result = formatter.format(&entry);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        // 应该包含 ANSI 颜色代码
        assert!(formatted.contains("\x1b[32m")); // 绿色（INFO级别）
        assert!(formatted.contains("\x1b[0m"));  // 重置颜色
    }
    
    #[test]
    fn test_compact_formatter() {
        let formatter = CompactFormatter::new();
        let entry = create_test_entry();
        
        let result = formatter.format(&entry);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        assert!(formatted.contains("I ")); // 级别简化为单字符
        assert!(formatted.contains("Test log message"));
        // 紧凑格式应该更短
        assert!(formatted.len() < 200);
    }
    
    #[test]
    fn test_csv_formatter() {
        let formatter = CsvFormatter::new();
        let entry = create_test_entry();
        
        let result = formatter.format(&entry);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        
        // 第一次调用应该包含标题行
        let lines: Vec<&str> = formatted.lines().collect();
        assert_eq!(lines.len(), 2); // 标题行 + 数据行
        assert!(lines[0].contains("timestamp,level,module"));
        assert!(lines[1].contains("INFO"));
        assert!(lines[1].contains("test_module"));
    }
    
    #[test]
    fn test_formatter_options() {
        let mut options = FormatterOptions::default();
        options.include_thread = true;
        options.max_message_length = Some(10);
        
        let formatter = HumanReadableFormatter::new().with_options(options);
        let entry = create_test_entry();
        
        let result = formatter.format(&entry);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        assert!(formatted.contains("[test_thread]")); // 包含线程ID
        assert!(formatted.contains("Test log m...")); // 消息被截断
    }
    
    #[test]
    fn test_formatter_factory() {
        let json_formatter = FormatterFactory::create("json");
        assert!(json_formatter.is_ok());
        assert_eq!(json_formatter.unwrap().name(), "json");
        
        let human_formatter = FormatterFactory::create("human");
        assert!(human_formatter.is_ok());
        assert_eq!(human_formatter.unwrap().name(), "human_readable");
        
        let invalid_formatter = FormatterFactory::create("invalid");
        assert!(invalid_formatter.is_err());
        
        let supported = FormatterFactory::supported_formatters();
        assert!(supported.contains(&"json"));
        assert!(supported.contains(&"human"));
        assert!(supported.contains(&"compact"));
    }
    
    #[test]
    fn test_csv_field_escaping() {
        let formatter = CsvFormatter::new();
        
        // 测试包含逗号和引号的消息
        let mut entry = create_test_entry();
        entry.message = "Message with, comma and \"quotes\"".to_string();
        
        let result = formatter.format(&entry);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        assert!(formatted.contains("\"Message with, comma and \"\"quotes\"\"\""));
    }
    
    #[test]
    fn test_message_length_limit() {
        let long_message = "A".repeat(1000);
        let mut entry = create_test_entry();
        entry.message = long_message;
        
        let mut options = FormatterOptions::default();
        options.max_message_length = Some(50);
        
        let formatter = JsonFormatter::new().with_options(options);
        let result = formatter.format(&entry);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        // 消息应该被截断并添加省略号
        assert!(formatted.contains("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA..."));
    }
}