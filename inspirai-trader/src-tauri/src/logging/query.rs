use std::collections::{HashMap, BTreeMap};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use regex::Regex;

use super::{
    config::{LogConfig, LogType, LogLevel},
    error::LogError,
    LogEntry,
};

/// 日志查询接口
#[derive(Debug)]
pub struct LogQueryEngine {
    config: LogConfig,
    index_manager: LogIndexManager,
}

impl LogQueryEngine {
    /// 创建新的查询引擎
    pub fn new(config: LogConfig) -> Result<Self, LogError> {
        let index_manager = LogIndexManager::new(&config)?;
        
        Ok(Self {
            config,
            index_manager,
        })
    }
    
    /// 执行日志查询
    pub async fn query(&self, query: LogQuery) -> Result<QueryResult, LogError> {
        // 验证查询参数
        query.validate()?;
        
        // 根据时间范围和日志类型确定需要搜索的文件
        let candidate_files = self.get_candidate_files(&query).await?;
        let files_searched = candidate_files.len();
        
        // 执行搜索
        let mut results = Vec::new();
        let mut total_scanned = 0;
        
        for file_info in candidate_files {
            match self.search_file(&file_info.path, &query).await {
                Ok(mut file_results) => {
                    total_scanned += file_results.len();
                    results.append(&mut file_results);
                    
                    // 检查结果数量限制
                    if results.len() >= query.limit {
                        results.truncate(query.limit);
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        file = %file_info.path.display(),
                        error = %e,
                        "搜索文件时出错"
                    );
                    continue;
                }
            }
        }
        
        // 排序结果
        self.sort_results(&mut results, &query);
        
        Ok(QueryResult {
            entries: results,
            total_found: total_scanned,
            query: query.clone(),
            execution_time_ms: 0, // TODO: 实际测量执行时间
            files_searched,
        })
    }
    
    /// 获取候选文件列表
    async fn get_candidate_files(&self, query: &LogQuery) -> Result<Vec<FileInfo>, LogError> {
        let mut files = Vec::new();
        
        // 获取指定日志类型的文件
        let log_types = if query.log_types.is_empty() {
            LogType::all()
        } else {
            query.log_types.clone()
        };
        
        for log_type in log_types {
            let log_dir = self.config.output_dir.join(log_type.as_str());
            
            if log_dir.exists() {
                let dir_files = self.scan_log_directory(&log_dir, &query.time_range).await?;
                files.extend(dir_files);
            }
        }
        
        // 按时间排序
        files.sort_by(|a, b| b.modified_time.cmp(&a.modified_time));
        
        Ok(files)
    }
    
    /// 扫描日志目录
    async fn scan_log_directory(
        &self,
        dir_path: &Path,
        time_range: &Option<TimeRange>,
    ) -> Result<Vec<FileInfo>, LogError> {
        let mut files = Vec::new();
        
        let entries = fs::read_dir(dir_path).map_err(LogError::WriteError)?;
        
        for entry in entries {
            let entry = entry.map_err(LogError::WriteError)?;
            let path = entry.path();
            
            if path.is_file() {
                let metadata = entry.metadata().map_err(LogError::WriteError)?;
                let modified_time = DateTime::<Utc>::from(
                    metadata.modified().map_err(LogError::WriteError)?
                );
                
                // 检查时间范围过滤
                if let Some(range) = time_range {
                    if !range.contains(modified_time) {
                        continue;
                    }
                }
                
                let is_compressed = path.extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "gz")
                    .unwrap_or(false);
                    
                files.push(FileInfo {
                    path,
                    size: metadata.len(),
                    modified_time,
                    is_compressed,
                });
            }
        }
        
        Ok(files)
    }
    
    /// 搜索单个文件
    async fn search_file(&self, file_path: &Path, query: &LogQuery) -> Result<Vec<LogEntry>, LogError> {
        let file_path_owned = file_path.to_owned();
        let query_owned = query.clone();
        
        // 在后台线程中执行文件搜索
        let results = tokio::task::spawn_blocking(move || {
            Self::search_file_sync(&file_path_owned, &query_owned)
        }).await
        .map_err(|e| LogError::QueryError {
            query: format!("搜索文件 {:?}", file_path),
        })?;
        
        results
    }
    
    /// 同步搜索文件
    fn search_file_sync(file_path: &Path, query: &LogQuery) -> Result<Vec<LogEntry>, LogError> {
        let mut results = Vec::new();
        
        // 判断是否为压缩文件
        let is_compressed = file_path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s == "gz")
            .unwrap_or(false);
        
        if is_compressed {
            // 处理压缩文件
            use flate2::read::GzDecoder;
            let file = fs::File::open(file_path).map_err(LogError::WriteError)?;
            let decoder = GzDecoder::new(file);
            let reader = BufReader::new(decoder);
            
            for (line_number, line_result) in reader.lines().enumerate() {
                let line = line_result.map_err(LogError::WriteError)?;
                
                if let Some(entry) = Self::parse_log_line(&line, line_number + 1)? {
                    if Self::matches_query(&entry, query) {
                        results.push(entry);
                        
                        if results.len() >= query.limit {
                            break;
                        }
                    }
                }
            }
        } else {
            // 处理普通文件
            let file = fs::File::open(file_path).map_err(LogError::WriteError)?;
            let reader = BufReader::new(file);
            
            for (line_number, line_result) in reader.lines().enumerate() {
                let line = line_result.map_err(LogError::WriteError)?;
                
                if let Some(entry) = Self::parse_log_line(&line, line_number + 1)? {
                    if Self::matches_query(&entry, query) {
                        results.push(entry);
                        
                        if results.len() >= query.limit {
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    /// 解析日志行
    fn parse_log_line(line: &str, line_number: usize) -> Result<Option<LogEntry>, LogError> {
        // 尝试解析 JSON 格式
        if line.trim().starts_with('{') {
            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(json) => {
                    // 从 JSON 构建 LogEntry
                    return Ok(Some(Self::parse_json_log_entry(&json)?));
                }
                Err(_) => {
                    // 如果 JSON 解析失败，尝试其他格式
                }
            }
        }
        
        // 尝试解析人类可读格式
        if let Some(entry) = Self::parse_human_readable_log(&line, line_number)? {
            return Ok(Some(entry));
        }
        
        // 如果都解析失败，跳过这一行
        Ok(None)
    }
    
    /// 从 JSON 解析日志条目
    fn parse_json_log_entry(json: &serde_json::Value) -> Result<LogEntry, LogError> {
        let timestamp = json.get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        
        let level = json.get("level")
            .and_then(|v| v.as_str())
            .and_then(|s| LogLevel::from_str(s).ok())
            .unwrap_or(LogLevel::Info);
        
        let module = json.get("module")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let thread_id = json.get("thread")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let message = json.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let request_id = json.get("request_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let session_id = json.get("session_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // 提取其他字段
        let mut fields = HashMap::new();
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if !["timestamp", "level", "module", "thread", "message", "request_id", "session_id"].contains(&key.as_str()) {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }
        
        let context = super::context::LogContext {
            timestamp,
            level,
            module: module.clone(),
            thread_id: thread_id.clone(),
            request_id: request_id.clone(),
            user_id: fields.get("user_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
            session_id: session_id.clone(),
            extra: fields.clone(),
        };
        
        Ok(LogEntry {
            timestamp,
            level,
            module,
            thread_id,
            message,
            context,
            request_id,
            session_id,
            fields,
        })
    }
    
    /// 解析人类可读格式的日志
    fn parse_human_readable_log(line: &str, _line_number: usize) -> Result<Option<LogEntry>, LogError> {
        // 这是一个简化的实现，实际应该根据具体的日志格式来解析
        // 例如：2024-01-15 18:30:45.123 [INFO ] [trading_service] 订单提交成功 ...
        
        let re = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}) \[(\w+)\s*\] \[([^\]]+)\] (.*)").unwrap();
        
        if let Some(captures) = re.captures(line) {
            let timestamp_str = captures.get(1).unwrap().as_str();
            let level_str = captures.get(2).unwrap().as_str();
            let module_str = captures.get(3).unwrap().as_str();
            let message_str = captures.get(4).unwrap().as_str();
            
            let timestamp = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S%.3f")
                .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                .unwrap_or_else(|_| Utc::now());
            
            let level = LogLevel::from_str(level_str).unwrap_or(LogLevel::Info);
            
            let context = super::context::LogContext {
                timestamp,
                level,
                module: module_str.to_string(),
                thread_id: "unknown".to_string(),
                request_id: None,
                user_id: None,
                session_id: None,
                extra: HashMap::new(),
            };
            
            return Ok(Some(LogEntry {
                timestamp,
                level,
                module: module_str.to_string(),
                thread_id: "unknown".to_string(),
                message: message_str.to_string(),
                context,
                request_id: None,
                session_id: None,
                fields: HashMap::new(),
            }));
        }
        
        Ok(None)
    }
    
    /// 检查日志条目是否匹配查询条件
    fn matches_query(entry: &LogEntry, query: &LogQuery) -> bool {
        // 检查日志级别
        if !query.levels.is_empty() && !query.levels.contains(&entry.level) {
            return false;
        }
        
        // 检查模块过滤
        if !query.modules.is_empty() {
            let matches_module = query.modules.iter().any(|module| {
                entry.module.contains(module)
            });
            if !matches_module {
                return false;
            }
        }
        
        // 检查关键字搜索
        if !query.keywords.is_empty() {
            let text_to_search = format!("{} {}", entry.message, 
                entry.fields.values()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            ).to_lowercase();
            
            let matches_keywords = query.keywords.iter().all(|keyword| {
                text_to_search.contains(&keyword.to_lowercase())
            });
            if !matches_keywords {
                return false;
            }
        }
        
        // 检查字段过滤
        for (field, expected_value) in &query.field_filters {
            match entry.fields.get(field) {
                Some(actual_value) => {
                    if actual_value.to_string() != *expected_value {
                        return false;
                    }
                }
                None => return false,
            }
        }
        
        // 检查时间范围
        if let Some(time_range) = &query.time_range {
            if !time_range.contains(entry.timestamp) {
                return false;
            }
        }
        
        true
    }
    
    /// 排序查询结果
    fn sort_results(&self, results: &mut Vec<LogEntry>, query: &LogQuery) {
        match query.sort_by {
            SortBy::Timestamp => {
                if query.sort_order == SortOrder::Descending {
                    results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                } else {
                    results.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                }
            }
            SortBy::Level => {
                if query.sort_order == SortOrder::Descending {
                    results.sort_by(|a, b| b.level.cmp(&a.level));
                } else {
                    results.sort_by(|a, b| a.level.cmp(&b.level));
                }
            }
            SortBy::Module => {
                if query.sort_order == SortOrder::Descending {
                    results.sort_by(|a, b| b.module.cmp(&a.module));
                } else {
                    results.sort_by(|a, b| a.module.cmp(&b.module));
                }
            }
        }
    }
    
    /// 重建索引
    pub async fn rebuild_index(&mut self) -> Result<(), LogError> {
        self.index_manager.rebuild(&self.config).await
    }
    
    /// 获取查询统计信息
    pub fn get_query_stats(&self) -> QueryStats {
        self.index_manager.get_stats()
    }
}

/// 日志查询条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQuery {
    /// 时间范围
    pub time_range: Option<TimeRange>,
    /// 日志级别过滤
    pub levels: Vec<LogLevel>,
    /// 日志类型过滤
    pub log_types: Vec<LogType>,
    /// 模块过滤
    pub modules: Vec<String>,
    /// 关键字搜索
    pub keywords: Vec<String>,
    /// 字段过滤
    pub field_filters: HashMap<String, String>,
    /// 排序方式
    pub sort_by: SortBy,
    /// 排序顺序
    pub sort_order: SortOrder,
    /// 结果数量限制
    pub limit: usize,
    /// 跳过条数
    pub offset: usize,
}

impl LogQuery {
    /// 创建新的查询
    pub fn new() -> Self {
        Self {
            time_range: None,
            levels: Vec::new(),
            log_types: Vec::new(),
            modules: Vec::new(),
            keywords: Vec::new(),
            field_filters: HashMap::new(),
            sort_by: SortBy::Timestamp,
            sort_order: SortOrder::Descending,
            limit: 1000,
            offset: 0,
        }
    }
    
    /// 设置时间范围
    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.time_range = Some(TimeRange { start, end });
        self
    }
    
    /// 添加日志级别过滤
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.levels.push(level);
        self
    }
    
    /// 添加日志类型过滤
    pub fn with_log_type(mut self, log_type: LogType) -> Self {
        self.log_types.push(log_type);
        self
    }
    
    /// 添加模块过滤
    pub fn with_module(mut self, module: &str) -> Self {
        self.modules.push(module.to_string());
        self
    }
    
    /// 添加关键字搜索
    pub fn with_keyword(mut self, keyword: &str) -> Self {
        self.keywords.push(keyword.to_string());
        self
    }
    
    /// 添加字段过滤
    pub fn with_field(mut self, field: &str, value: &str) -> Self {
        self.field_filters.insert(field.to_string(), value.to_string());
        self
    }
    
    /// 设置排序
    pub fn with_sort(mut self, sort_by: SortBy, sort_order: SortOrder) -> Self {
        self.sort_by = sort_by;
        self.sort_order = sort_order;
        self
    }
    
    /// 设置限制
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
    
    /// 设置偏移
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }
    
    /// 验证查询参数
    pub fn validate(&self) -> Result<(), LogError> {
        if self.limit == 0 {
            return Err(LogError::QueryError {
                query: "limit 必须大于 0".to_string(),
            });
        }
        
        if self.limit > 10000 {
            return Err(LogError::QueryError {
                query: "limit 不能超过 10000".to_string(),
            });
        }
        
        if let Some(time_range) = &self.time_range {
            if time_range.start >= time_range.end {
                return Err(LogError::QueryError {
                    query: "开始时间必须早于结束时间".to_string(),
                });
            }
        }
        
        Ok(())
    }
}

impl Default for LogQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeRange {
    /// 检查时间戳是否在范围内
    pub fn contains(&self, timestamp: DateTime<Utc>) -> bool {
        timestamp >= self.start && timestamp <= self.end
    }
    
    /// 创建最近N小时的时间范围
    pub fn last_hours(hours: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::hours(hours);
        Self { start, end }
    }
    
    /// 创建最近N天的时间范围
    pub fn last_days(days: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::days(days);
        Self { start, end }
    }
    
    /// 创建今天的时间范围
    pub fn today() -> Self {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0)
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            .unwrap_or(now);
        let end = now;
        Self { start, end }
    }
}

/// 排序方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortBy {
    Timestamp,
    Level,
    Module,
}

/// 排序顺序
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub entries: Vec<LogEntry>,
    pub total_found: usize,
    pub query: LogQuery,
    pub execution_time_ms: u64,
    pub files_searched: usize,
}

/// 文件信息
#[derive(Debug, Clone)]
struct FileInfo {
    path: PathBuf,
    size: u64,
    modified_time: DateTime<Utc>,
    is_compressed: bool,
}

/// 日志索引管理器
#[derive(Debug)]
pub struct LogIndexManager {
    indices: BTreeMap<String, LogIndex>,
    stats: QueryStats,
}

impl LogIndexManager {
    /// 创建新的索引管理器
    pub fn new(config: &LogConfig) -> Result<Self, LogError> {
        let mut manager = Self {
            indices: BTreeMap::new(),
            stats: QueryStats::default(),
        };
        
        // 加载现有索引
        manager.load_indices(config)?;
        
        Ok(manager)
    }
    
    /// 加载索引
    fn load_indices(&mut self, config: &LogConfig) -> Result<(), LogError> {
        let index_file = config.output_dir.join("log_index.json");
        
        if index_file.exists() {
            let content = fs::read_to_string(&index_file).map_err(LogError::WriteError)?;
            let indices: BTreeMap<String, LogIndex> = serde_json::from_str(&content)
                .map_err(LogError::SerializationError)?;
            
            self.indices = indices;
        }
        
        Ok(())
    }
    
    /// 保存索引
    fn save_indices(&self, config: &LogConfig) -> Result<(), LogError> {
        let index_file = config.output_dir.join("log_index.json");
        
        let content = serde_json::to_string_pretty(&self.indices)
            .map_err(LogError::SerializationError)?;
        
        fs::write(&index_file, content).map_err(LogError::WriteError)?;
        
        Ok(())
    }
    
    /// 重建索引
    pub async fn rebuild(&mut self, config: &LogConfig) -> Result<(), LogError> {
        self.indices.clear();
        
        for log_type in LogType::all() {
            let log_dir = config.output_dir.join(log_type.as_str());
            
            if log_dir.exists() {
                self.index_directory(&log_dir).await?;
            }
        }
        
        self.save_indices(config)?;
        self.stats.total_indices = self.indices.len();
        
        Ok(())
    }
    
    /// 索引目录
    async fn index_directory(&mut self, dir_path: &Path) -> Result<(), LogError> {
        let entries = fs::read_dir(dir_path).map_err(LogError::WriteError)?;
        
        for entry in entries {
            let entry = entry.map_err(LogError::WriteError)?;
            let path = entry.path();
            
            if path.is_file() {
                self.index_file(&path).await?;
            }
        }
        
        Ok(())
    }
    
    /// 索引单个文件
    async fn index_file(&mut self, file_path: &Path) -> Result<(), LogError> {
        let metadata = fs::metadata(file_path).map_err(LogError::WriteError)?;
        let modified_time = DateTime::<Utc>::from(
            metadata.modified().map_err(LogError::WriteError)?
        );
        
        // 计算文件校验和
        let checksum = self.calculate_file_checksum(file_path)?;
        
        let index = LogIndex {
            file_path: file_path.to_path_buf(),
            start_time: modified_time, // 简化实现，实际应该读取文件内容获取
            end_time: modified_time,
            log_count: 0, // 简化实现
            size_bytes: metadata.len(),
            checksum,
        };
        
        let key = file_path.to_string_lossy().to_string();
        self.indices.insert(key, index);
        
        Ok(())
    }
    
    /// 计算文件校验和
    fn calculate_file_checksum(&self, file_path: &Path) -> Result<String, LogError> {
        use sha2::{Sha256, Digest};
        
        let content = fs::read(file_path).map_err(LogError::WriteError)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> QueryStats {
        self.stats.clone()
    }
}

/// 日志索引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIndex {
    pub file_path: PathBuf,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub log_count: u64,
    pub size_bytes: u64,
    pub checksum: String,
}

/// 查询统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryStats {
    pub total_queries: u64,
    pub total_indices: usize,
    pub average_query_time_ms: f64,
    pub cache_hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;

    fn create_test_config() -> (LogConfig, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = LogConfig {
            output_dir: temp_dir.path().to_path_buf(),
            ..LogConfig::development()
        };
        (config, temp_dir)
    }
    
    fn create_test_log_file(path: &Path, entries: &[&str]) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut file = fs::File::create(path)?;
        for entry in entries {
            writeln!(file, "{}", entry)?;
        }
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_log_query_engine_creation() {
        let (config, _temp_dir) = create_test_config();
        config.ensure_directories().unwrap();
        
        let engine = LogQueryEngine::new(config);
        assert!(engine.is_ok());
    }
    
    #[tokio::test]
    async fn test_json_log_parsing() {
        let json_line = r#"{"timestamp":"2024-01-15T10:30:45.123Z","level":"INFO","module":"test_module","message":"test message","request_id":"req_123"}"#;
        
        let entry = LogQueryEngine::parse_log_line(json_line, 1).unwrap();
        assert!(entry.is_some());
        
        let entry = entry.unwrap();
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.module, "test_module");
        assert_eq!(entry.message, "test message");
        assert_eq!(entry.request_id, Some("req_123".to_string()));
    }
    
    #[tokio::test]
    async fn test_human_readable_log_parsing() {
        let log_line = "2024-01-15 18:30:45.123 [INFO ] [trading_service] 订单提交成功";
        
        let entry = LogQueryEngine::parse_log_line(log_line, 1).unwrap();
        assert!(entry.is_some());
        
        let entry = entry.unwrap();
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.module, "trading_service");
        assert_eq!(entry.message, "订单提交成功");
    }
    
    #[tokio::test]
    async fn test_query_validation() {
        let mut query = LogQuery::new();
        query.limit = 0;
        
        let result = query.validate();
        assert!(result.is_err());
        
        query.limit = 1000;
        let result = query.validate();
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_time_range() {
        let range = TimeRange::last_hours(1);
        let now = Utc::now();
        let one_hour_ago = now - chrono::Duration::hours(1);
        let two_hours_ago = now - chrono::Duration::hours(2);
        
        assert!(range.contains(one_hour_ago));
        assert!(!range.contains(two_hours_ago));
        
        let today_range = TimeRange::today();
        assert!(today_range.contains(now));
    }
    
    #[tokio::test]
    async fn test_query_builder() {
        let query = LogQuery::new()
            .with_level(LogLevel::Error)
            .with_log_type(LogType::Trading)
            .with_module("order_service")
            .with_keyword("失败")
            .with_field("account_id", "12345")
            .with_limit(500);
        
        assert_eq!(query.levels, vec![LogLevel::Error]);
        assert_eq!(query.log_types, vec![LogType::Trading]);
        assert_eq!(query.modules, vec!["order_service".to_string()]);
        assert_eq!(query.keywords, vec!["失败".to_string()]);
        assert_eq!(query.field_filters.get("account_id"), Some(&"12345".to_string()));
        assert_eq!(query.limit, 500);
    }
    
    #[tokio::test]
    async fn test_log_search() {
        let (config, _temp_dir) = create_test_config();
        config.ensure_directories().unwrap();
        
        // 创建测试日志文件
        let log_file = config.get_log_file_path(LogType::App);
        let test_entries = vec![
            r#"{"timestamp":"2024-01-15T10:30:45.123Z","level":"INFO","module":"test_module","message":"正常消息"}"#,
            r#"{"timestamp":"2024-01-15T10:30:46.123Z","level":"ERROR","module":"test_module","message":"错误消息"}"#,
            r#"{"timestamp":"2024-01-15T10:30:47.123Z","level":"INFO","module":"other_module","message":"其他消息"}"#,
        ];
        create_test_log_file(&log_file, &test_entries).unwrap();
        
        let engine = LogQueryEngine::new(config).unwrap();
        
        // 测试级别过滤
        let query = LogQuery::new().with_level(LogLevel::Error);
        let result = engine.query(query).await.unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].message, "错误消息");
        
        // 测试模块过滤
        let query = LogQuery::new().with_module("other_module");
        let result = engine.query(query).await.unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].message, "其他消息");
        
        // 测试关键字搜索
        let query = LogQuery::new().with_keyword("正常");
        let result = engine.query(query).await.unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].message, "正常消息");
    }
    
    #[tokio::test]
    async fn test_index_manager() {
        let (config, _temp_dir) = create_test_config();
        config.ensure_directories().unwrap();
        
        let mut index_manager = LogIndexManager::new(&config).unwrap();
        
        // 创建一个测试文件
        let log_file = config.get_log_file_path(LogType::App);
        create_test_log_file(&log_file, &["test log entry"]).unwrap();
        
        // 重建索引
        let result = index_manager.rebuild(&config).await;
        assert!(result.is_ok());
        
        let stats = index_manager.get_stats();
        assert!(stats.total_indices > 0);
    }
}