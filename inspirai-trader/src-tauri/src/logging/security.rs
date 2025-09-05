use std::collections::HashMap;
use regex::Regex;
use serde::{Serialize, Deserialize};
use tokio::io::AsyncWriteExt;

use super::{LogEntry, error::LogError};

/// 数据脱敏器 - 负责处理敏感信息的掩码和脱敏
#[derive(Debug)]
pub struct DataMasker {
    patterns: Vec<MaskPattern>,
    field_rules: HashMap<String, MaskType>,
    enabled: bool,
}

impl DataMasker {
    /// 创建新的数据脱敏器
    pub fn new() -> Self {
        let mut masker = Self {
            patterns: Vec::new(),
            field_rules: HashMap::new(),
            enabled: true,
        };
        
        // 初始化默认脱敏规则
        masker.init_default_patterns();
        
        masker
    }
    
    /// 初始化默认脱敏模式
    fn init_default_patterns(&mut self) {
        // 密码字段脱敏
        self.add_field_rule("password", MaskType::FullMask);
        self.add_field_rule("passwd", MaskType::FullMask);
        self.add_field_rule("pwd", MaskType::FullMask);
        self.add_field_rule("secret", MaskType::FullMask);
        self.add_field_rule("token", MaskType::FullMask);
        self.add_field_rule("key", MaskType::FullMask);
        
        // 资金相关字段脱敏
        self.add_field_rule("balance", MaskType::PartialMask(2));
        self.add_field_rule("amount", MaskType::PartialMask(2));
        self.add_field_rule("fund", MaskType::PartialMask(2));
        self.add_field_rule("money", MaskType::PartialMask(2));
        
        // 用户标识符部分脱敏
        self.add_field_rule("user_id", MaskType::PartialMask(4));
        self.add_field_rule("account_id", MaskType::PartialMask(4));
        self.add_field_rule("client_id", MaskType::PartialMask(4));
        
        // 身份证号脱敏
        self.add_field_rule("id_card", MaskType::PartialMask(4));
        self.add_field_rule("identity", MaskType::PartialMask(4));
        
        // 电话号码脱敏
        self.add_field_rule("phone", MaskType::PartialMask(3));
        self.add_field_rule("mobile", MaskType::PartialMask(3));
        self.add_field_rule("tel", MaskType::PartialMask(3));
        
        // 添加正则表达式模式脱敏
        self.add_regex_pattern(
            "密码模式",
            r"(?i)(password|passwd|pwd)\s*[:=]\s*([^\s,}]+)",
            MaskType::FullMask
        );
        
        self.add_regex_pattern(
            "身份证号模式",
            r"\b\d{17}[\dXx]\b",
            MaskType::PartialMask(4)
        );
        
        self.add_regex_pattern(
            "手机号模式",
            r"\b1[3-9]\d{9}\b",
            MaskType::PartialMask(3)
        );
        
        self.add_regex_pattern(
            "银行卡号模式",
            r"\b\d{16,19}\b",
            MaskType::PartialMask(4)
        );
        
        self.add_regex_pattern(
            "邮箱模式",
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b",
            MaskType::PartialMask(3)
        );
    }
    
    /// 添加字段规则
    pub fn add_field_rule(&mut self, field_name: &str, mask_type: MaskType) {
        self.field_rules.insert(field_name.to_string(), mask_type);
    }
    
    /// 添加正则表达式模式
    pub fn add_regex_pattern(&mut self, name: &str, pattern: &str, mask_type: MaskType) {
        if let Ok(regex) = Regex::new(pattern) {
            self.patterns.push(MaskPattern {
                name: name.to_string(),
                regex,
                mask_type,
            });
        }
    }
    
    /// 移除字段规则
    pub fn remove_field_rule(&mut self, field_name: &str) {
        self.field_rules.remove(field_name);
    }
    
    /// 启用或禁用脱敏
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// 脱敏日志条目
    pub fn mask_log_entry(&self, entry: &mut LogEntry) -> Result<(), LogError> {
        if !self.enabled {
            return Ok(());
        }
        
        // 脱敏消息内容
        entry.message = self.mask_text(&entry.message);
        
        // 脱敏字段
        for (field_name, value) in entry.fields.iter_mut() {
            if let Some(mask_type) = self.field_rules.get(field_name) {
                *value = self.mask_json_value(value, mask_type);
            } else {
                // 检查值的内容是否匹配脱敏模式
                if let Some(str_value) = value.as_str() {
                    let masked = self.mask_text(str_value);
                    if masked != str_value {
                        *value = serde_json::Value::String(masked);
                    }
                }
            }
        }
        
        // 脱敏上下文中的额外字段
        for (field_name, value) in entry.context.extra.iter_mut() {
            if let Some(mask_type) = self.field_rules.get(field_name) {
                *value = self.mask_json_value(value, mask_type);
            } else {
                if let Some(str_value) = value.as_str() {
                    let masked = self.mask_text(str_value);
                    if masked != str_value {
                        *value = serde_json::Value::String(masked);
                    }
                }
            }
        }
        
        // 脱敏用户ID等敏感标识符
        if let Some(user_id) = &entry.context.user_id {
            entry.context.user_id = Some(self.mask_string(user_id, &MaskType::PartialMask(4)));
        }
        
        Ok(())
    }
    
    /// 脱敏文本内容
    fn mask_text(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        for pattern in &self.patterns {
            result = pattern.regex.replace_all(&result, |caps: &regex::Captures| {
                if caps.len() >= 2 {
                    // 如果有捕获组，只脱敏捕获的部分
                    let matched = caps.get(2).unwrap_or(caps.get(0).unwrap()).as_str();
                    let masked = self.mask_string(matched, &pattern.mask_type);
                    result.replace(matched, &masked)
                } else {
                    // 否则脱敏整个匹配
                    let matched = caps.get(0).unwrap().as_str();
                    self.mask_string(matched, &pattern.mask_type)
                }
            }).to_string();
        }
        
        result
    }
    
    /// 脱敏JSON值
    fn mask_json_value(&self, value: &serde_json::Value, mask_type: &MaskType) -> serde_json::Value {
        match value {
            serde_json::Value::String(s) => {
                serde_json::Value::String(self.mask_string(s, mask_type))
            }
            serde_json::Value::Number(n) => {
                // 数字脱敏：转换为字符串后脱敏
                let str_value = n.to_string();
                serde_json::Value::String(self.mask_string(&str_value, mask_type))
            }
            _ => value.clone(), // 其他类型不脱敏
        }
    }
    
    /// 脱敏字符串
    fn mask_string(&self, text: &str, mask_type: &MaskType) -> String {
        match mask_type {
            MaskType::FullMask => "*".repeat(std::cmp::min(text.len(), 8)),
            MaskType::PartialMask(keep_chars) => {
                if text.len() <= *keep_chars {
                    "*".repeat(text.len())
                } else {
                    let prefix = &text[..*keep_chars];
                    let suffix_len = std::cmp::min(2, text.len() - keep_chars);
                    let suffix = &text[text.len() - suffix_len..];
                    let mask_count = text.len() - keep_chars - suffix_len;
                    format!("{}{}{}", prefix, "*".repeat(mask_count), suffix)
                }
            }
            MaskType::HashMask => {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(text.as_bytes());
                format!("hash:{:x}", hasher.finalize())[..16].to_string()
            }
            MaskType::Truncate(max_len) => {
                if text.len() > *max_len {
                    format!("{}...", &text[..*max_len])
                } else {
                    text.to_string()
                }
            }
        }
    }
    
    /// 检查文本是否包含敏感信息
    pub fn contains_sensitive_data(&self, text: &str) -> bool {
        for pattern in &self.patterns {
            if pattern.regex.is_match(text) {
                return true;
            }
        }
        false
    }
    
    /// 获取脱敏统计
    pub fn get_stats(&self) -> MaskerStats {
        MaskerStats {
            enabled: self.enabled,
            pattern_count: self.patterns.len(),
            field_rule_count: self.field_rules.len(),
        }
    }
}

impl Default for DataMasker {
    fn default() -> Self {
        Self::new()
    }
}

/// 脱敏模式
#[derive(Debug, Clone)]
pub struct MaskPattern {
    pub name: String,
    pub regex: Regex,
    pub mask_type: MaskType,
}

/// 脱敏类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaskType {
    /// 完全隐藏
    FullMask,
    /// 部分隐藏，保留前N位
    PartialMask(usize),
    /// 哈希处理
    HashMask,
    /// 截断到N位
    Truncate(usize),
}

/// 脱敏器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskerStats {
    pub enabled: bool,
    pub pattern_count: usize,
    pub field_rule_count: usize,
}

/// 访问控制器
#[derive(Debug)]
pub struct AccessController {
    permissions: HashMap<String, Vec<Permission>>,
    enabled: bool,
}

impl AccessController {
    /// 创建新的访问控制器
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
            enabled: true,
        }
    }
    
    /// 添加权限
    pub fn add_permission(&mut self, user_id: &str, permission: Permission) {
        self.permissions
            .entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(permission);
    }
    
    /// 检查权限
    pub fn check_permission(&self, user_id: &str, required_permission: &Permission) -> bool {
        if !self.enabled {
            return true;
        }
        
        if let Some(user_permissions) = self.permissions.get(user_id) {
            user_permissions.iter().any(|p| p.allows(required_permission))
        } else {
            false
        }
    }
    
    /// 过滤日志条目（基于权限）
    pub fn filter_log_entries(
        &self,
        entries: Vec<LogEntry>,
        user_id: &str,
    ) -> Vec<LogEntry> {
        if !self.enabled {
            return entries;
        }
        
        entries.into_iter()
            .filter(|entry| {
                // 检查是否有读取该日志类型的权限
                let log_type = self.determine_log_type(entry);
                let required_permission = Permission::ReadLogs(log_type);
                self.check_permission(user_id, &required_permission)
            })
            .collect()
    }
    
    /// 确定日志条目的类型
    fn determine_log_type(&self, entry: &LogEntry) -> super::config::LogType {
        // 简化的日志类型确定逻辑
        if entry.module.contains("ctp") {
            super::config::LogType::Ctp
        } else if entry.module.contains("trading") {
            super::config::LogType::Trading
        } else if entry.module.contains("market_data") {
            super::config::LogType::MarketData
        } else if entry.level >= super::config::LogLevel::Error {
            super::config::LogType::Error
        } else {
            super::config::LogType::App
        }
    }
    
    /// 启用或禁用访问控制
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// 获取用户权限列表
    pub fn get_user_permissions(&self, user_id: &str) -> Vec<Permission> {
        self.permissions.get(user_id).cloned().unwrap_or_default()
    }
    
    /// 移除用户权限
    pub fn remove_user_permissions(&mut self, user_id: &str) {
        self.permissions.remove(user_id);
    }
}

impl Default for AccessController {
    fn default() -> Self {
        Self::new()
    }
}

/// 权限类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    /// 读取日志
    ReadLogs(super::config::LogType),
    /// 查询日志
    QueryLogs,
    /// 导出日志
    ExportLogs,
    /// 管理员权限
    Admin,
    /// 查看系统指标
    ViewMetrics,
    /// 管理日志配置
    ManageConfig,
}

impl Permission {
    /// 检查权限是否允许指定操作
    pub fn allows(&self, required: &Permission) -> bool {
        match (self, required) {
            // 管理员权限允许所有操作
            (Permission::Admin, _) => true,
            
            // 精确匹配
            (a, b) if a == b => true,
            
            // 查询日志权限包含读取所有类型日志的权限
            (Permission::QueryLogs, Permission::ReadLogs(_)) => true,
            
            // 导出日志权限包含查询和读取权限
            (Permission::ExportLogs, Permission::QueryLogs) => true,
            (Permission::ExportLogs, Permission::ReadLogs(_)) => true,
            
            _ => false,
        }
    }
}

/// 安全审计器
#[derive(Debug)]
pub struct SecurityAuditor {
    enabled: bool,
    audit_log_path: Option<std::path::PathBuf>,
}

impl SecurityAuditor {
    /// 创建新的安全审计器
    pub fn new() -> Self {
        Self {
            enabled: true,
            audit_log_path: None,
        }
    }
    
    /// 设置审计日志路径
    pub fn with_audit_log(mut self, path: std::path::PathBuf) -> Self {
        self.audit_log_path = Some(path);
        self
    }
    
    /// 启用或禁用审计
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// 记录审计事件
    pub async fn audit_event(&self, event: AuditEvent) -> Result<(), LogError> {
        if !self.enabled {
            return Ok(());
        }
        
        // 创建审计记录
        let audit_record = AuditRecord {
            timestamp: chrono::Utc::now(),
            event,
            source_ip: self.get_source_ip().await,
            user_agent: self.get_user_agent().await,
        };
        
        // 记录到审计日志
        tracing::info!(
            audit = true,
            event_type = audit_record.event.event_type(),
            user_id = audit_record.event.user_id(),
            resource = audit_record.event.resource(),
            success = audit_record.event.success(),
            source_ip = audit_record.source_ip,
            "安全审计事件"
        );
        
        // 如果配置了审计日志文件，写入文件
        if let Some(audit_path) = &self.audit_log_path {
            self.write_audit_record(audit_path, &audit_record).await?;
        }
        
        Ok(())
    }
    
    /// 写入审计记录到文件
    async fn write_audit_record(
        &self,
        path: &std::path::Path,
        record: &AuditRecord,
    ) -> Result<(), LogError> {
        let json = serde_json::to_string(record)
            .map_err(LogError::SerializationError)?;
        
        let content = format!("{}\n", json);
        
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
            .map_err(LogError::WriteError)?;
        
        file.write_all(content.as_bytes())
            .await
            .map_err(LogError::WriteError)?;
        
        Ok(())
    }
    
    /// 获取源IP地址（简化实现）
    async fn get_source_ip(&self) -> Option<String> {
        // 实际实现中应该从请求上下文获取
        None
    }
    
    /// 获取用户代理（简化实现）
    async fn get_user_agent(&self) -> Option<String> {
        // 实际实现中应该从请求上下文获取
        None
    }
}

impl Default for SecurityAuditor {
    fn default() -> Self {
        Self::new()
    }
}

/// 审计事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEvent {
    /// 用户登录
    UserLogin {
        user_id: String,
        success: bool,
    },
    /// 用户登出
    UserLogout {
        user_id: String,
    },
    /// 日志查询
    LogQuery {
        user_id: String,
        query: String,
        result_count: usize,
    },
    /// 日志导出
    LogExport {
        user_id: String,
        log_types: Vec<String>,
        time_range: String,
    },
    /// 配置更改
    ConfigChange {
        user_id: String,
        config_key: String,
        old_value: Option<String>,
        new_value: String,
    },
    /// 权限更改
    PermissionChange {
        admin_user_id: String,
        target_user_id: String,
        permission: String,
        action: String, // "granted" | "revoked"
    },
    /// 文件访问
    FileAccess {
        user_id: String,
        file_path: String,
        action: String, // "read" | "write" | "delete"
        success: bool,
    },
}

impl AuditEvent {
    /// 获取事件类型
    pub fn event_type(&self) -> &'static str {
        match self {
            AuditEvent::UserLogin { .. } => "user_login",
            AuditEvent::UserLogout { .. } => "user_logout",
            AuditEvent::LogQuery { .. } => "log_query",
            AuditEvent::LogExport { .. } => "log_export",
            AuditEvent::ConfigChange { .. } => "config_change",
            AuditEvent::PermissionChange { .. } => "permission_change",
            AuditEvent::FileAccess { .. } => "file_access",
        }
    }
    
    /// 获取用户ID
    pub fn user_id(&self) -> &str {
        match self {
            AuditEvent::UserLogin { user_id, .. } => user_id,
            AuditEvent::UserLogout { user_id } => user_id,
            AuditEvent::LogQuery { user_id, .. } => user_id,
            AuditEvent::LogExport { user_id, .. } => user_id,
            AuditEvent::ConfigChange { user_id, .. } => user_id,
            AuditEvent::PermissionChange { admin_user_id, .. } => admin_user_id,
            AuditEvent::FileAccess { user_id, .. } => user_id,
        }
    }
    
    /// 获取资源标识
    pub fn resource(&self) -> String {
        match self {
            AuditEvent::UserLogin { .. } => "authentication".to_string(),
            AuditEvent::UserLogout { .. } => "authentication".to_string(),
            AuditEvent::LogQuery { query, .. } => format!("log_query:{}", query),
            AuditEvent::LogExport { log_types, .. } => format!("log_export:{:?}", log_types),
            AuditEvent::ConfigChange { config_key, .. } => format!("config:{}", config_key),
            AuditEvent::PermissionChange { target_user_id, permission, .. } => {
                format!("permission:{}:{}", target_user_id, permission)
            }
            AuditEvent::FileAccess { file_path, .. } => format!("file:{}", file_path),
        }
    }
    
    /// 获取操作是否成功
    pub fn success(&self) -> bool {
        match self {
            AuditEvent::UserLogin { success, .. } => *success,
            AuditEvent::FileAccess { success, .. } => *success,
            _ => true, // 其他事件默认认为成功
        }
    }
}

/// 审计记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event: AuditEvent,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
}

/// 安全管理器 - 集成所有安全组件
#[derive(Debug)]
pub struct SecurityManager {
    pub data_masker: DataMasker,
    pub access_controller: AccessController,
    pub auditor: SecurityAuditor,
    enabled: bool,
}

impl SecurityManager {
    /// 创建新的安全管理器
    pub fn new() -> Self {
        Self {
            data_masker: DataMasker::new(),
            access_controller: AccessController::new(),
            auditor: SecurityAuditor::new(),
            enabled: true,
        }
    }
    
    /// 启用或禁用安全功能
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.data_masker.set_enabled(enabled);
        self.access_controller.set_enabled(enabled);
        self.auditor.set_enabled(enabled);
    }
    
    /// 安全处理日志条目
    pub async fn secure_log_entry(
        &self,
        mut entry: LogEntry,
        user_id: Option<&str>,
    ) -> Result<LogEntry, LogError> {
        if !self.enabled {
            return Ok(entry);
        }
        
        // 数据脱敏
        self.data_masker.mask_log_entry(&mut entry)?;
        
        // 记录访问审计（如果有用户ID）
        if let Some(uid) = user_id {
            self.auditor.audit_event(AuditEvent::FileAccess {
                user_id: uid.to_string(),
                file_path: "log_entry".to_string(),
                action: "read".to_string(),
                success: true,
            }).await?;
        }
        
        Ok(entry)
    }
    
    /// 安全过滤日志条目列表
    pub async fn secure_filter_entries(
        &self,
        entries: Vec<LogEntry>,
        user_id: &str,
    ) -> Result<Vec<LogEntry>, LogError> {
        if !self.enabled {
            return Ok(entries);
        }
        
        // 权限过滤
        let filtered_entries = self.access_controller.filter_log_entries(entries, user_id);
        
        // 数据脱敏
        let mut secured_entries = Vec::new();
        for mut entry in filtered_entries {
            self.data_masker.mask_log_entry(&mut entry)?;
            secured_entries.push(entry);
        }
        
        // 记录查询审计
        self.auditor.audit_event(AuditEvent::LogQuery {
            user_id: user_id.to_string(),
            query: "filtered_query".to_string(),
            result_count: secured_entries.len(),
        }).await?;
        
        Ok(secured_entries)
    }
    
    /// 获取安全统计信息
    pub fn get_security_stats(&self) -> SecurityStats {
        SecurityStats {
            enabled: self.enabled,
            masker_stats: self.data_masker.get_stats(),
            total_users: self.access_controller.permissions.len(),
        }
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 安全统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    pub enabled: bool,
    pub masker_stats: MaskerStats,
    pub total_users: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_log_entry() -> LogEntry {
        let mut fields = HashMap::new();
        fields.insert("password".to_string(), serde_json::Value::String("secret123".to_string()));
        fields.insert("user_id".to_string(), serde_json::Value::String("user123456789".to_string()));
        fields.insert("balance".to_string(), serde_json::Value::Number(10000.50.into()));
        fields.insert("phone".to_string(), serde_json::Value::String("13812345678".to_string()));
        
        let context = super::super::context::LogContext {
            timestamp: chrono::Utc::now(),
            level: super::super::config::LogLevel::Info,
            module: "test_module".to_string(),
            thread_id: "test_thread".to_string(),
            request_id: None,
            user_id: Some("user123456789".to_string()),
            session_id: None,
            extra: fields.clone(),
        };
        
        LogEntry {
            timestamp: chrono::Utc::now(),
            level: super::super::config::LogLevel::Info,
            module: "test_module".to_string(),
            thread_id: "test_thread".to_string(),
            message: "用户 user123456789 使用密码 password123 登录".to_string(),
            context,
            request_id: None,
            session_id: None,
            fields,
        }
    }
    
    #[test]
    fn test_data_masker_creation() {
        let masker = DataMasker::new();
        assert!(masker.enabled);
        assert!(!masker.patterns.is_empty());
        assert!(!masker.field_rules.is_empty());
    }
    
    #[test]
    fn test_field_masking() {
        let masker = DataMasker::new();
        let mut entry = create_test_log_entry();
        
        masker.mask_log_entry(&mut entry).unwrap();
        
        // 检查密码字段是否被完全脱敏
        let password_field = entry.fields.get("password").unwrap();
        assert_eq!(password_field.as_str().unwrap(), "********");
        
        // 检查用户ID字段是否被部分脱敏
        let user_id_field = entry.fields.get("user_id").unwrap();
        let user_id_str = user_id_field.as_str().unwrap();
        assert!(user_id_str.starts_with("user"));
        assert!(user_id_str.contains("*"));
        assert!(user_id_str.len() > 8);
    }
    
    #[test]
    fn test_text_masking() {
        let masker = DataMasker::new();
        let original_text = "用户手机号是13812345678，请联系";
        let masked_text = masker.mask_text(original_text);
        
        // 手机号应该被脱敏
        assert_ne!(masked_text, original_text);
        assert!(!masked_text.contains("13812345678"));
        assert!(masked_text.contains("*"));
    }
    
    #[test]
    fn test_mask_types() {
        let masker = DataMasker::new();
        
        // 测试完全脱敏
        let full_masked = masker.mask_string("password123", &MaskType::FullMask);
        assert_eq!(full_masked, "********");
        
        // 测试部分脱敏
        let partial_masked = masker.mask_string("user123456789", &MaskType::PartialMask(4));
        assert!(partial_masked.starts_with("user"));
        assert!(partial_masked.contains("*"));
        assert!(partial_masked.ends_with("89"));
        
        // 测试哈希脱敏
        let hash_masked = masker.mask_string("sensitive_data", &MaskType::HashMask);
        assert!(hash_masked.starts_with("hash:"));
        assert_eq!(hash_masked.len(), 21); // "hash:" + 16个字符
        
        // 测试截断
        let truncated = masker.mask_string("very_long_text_here", &MaskType::Truncate(10));
        assert_eq!(truncated, "very_long_...");
    }
    
    #[test]
    fn test_access_controller() {
        let mut controller = AccessController::new();
        
        // 添加权限
        controller.add_permission("user1", Permission::ReadLogs(super::super::config::LogType::Trading));
        controller.add_permission("admin", Permission::Admin);
        
        // 测试权限检查
        assert!(controller.check_permission("admin", &Permission::ReadLogs(super::super::config::LogType::Trading)));
        assert!(controller.check_permission("user1", &Permission::ReadLogs(super::super::config::LogType::Trading)));
        assert!(!controller.check_permission("user1", &Permission::ReadLogs(super::super::config::LogType::Ctp)));
        assert!(!controller.check_permission("user2", &Permission::ReadLogs(super::super::config::LogType::Trading)));
    }
    
    #[test]
    fn test_permission_hierarchy() {
        let admin_perm = Permission::Admin;
        let query_perm = Permission::QueryLogs;
        let export_perm = Permission::ExportLogs;
        let read_perm = Permission::ReadLogs(super::super::config::LogType::App);
        
        // 测试权限层级
        assert!(admin_perm.allows(&query_perm));
        assert!(admin_perm.allows(&read_perm));
        assert!(query_perm.allows(&read_perm));
        assert!(export_perm.allows(&query_perm));
        assert!(export_perm.allows(&read_perm));
        
        // 测试不允许的权限
        assert!(!read_perm.allows(&query_perm));
        assert!(!query_perm.allows(&admin_perm));
    }
    
    #[tokio::test]
    async fn test_security_auditor() {
        let auditor = SecurityAuditor::new();
        
        let event = AuditEvent::UserLogin {
            user_id: "test_user".to_string(),
            success: true,
        };
        
        // 测试审计事件记录
        let result = auditor.audit_event(event).await;
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_audit_event_properties() {
        let login_event = AuditEvent::UserLogin {
            user_id: "test_user".to_string(),
            success: true,
        };
        
        assert_eq!(login_event.event_type(), "user_login");
        assert_eq!(login_event.user_id(), "test_user");
        assert_eq!(login_event.resource(), "authentication");
        assert!(login_event.success());
        
        let query_event = AuditEvent::LogQuery {
            user_id: "test_user".to_string(),
            query: "level=ERROR".to_string(),
            result_count: 10,
        };
        
        assert_eq!(query_event.event_type(), "log_query");
        assert_eq!(query_event.resource(), "log_query:level=ERROR");
    }
    
    #[tokio::test]
    async fn test_security_manager() {
        let mut security_manager = SecurityManager::new();
        
        // 添加权限
        security_manager.access_controller.add_permission(
            "test_user", 
            Permission::ReadLogs(super::super::config::LogType::App)
        );
        
        // 测试安全处理日志条目
        let entry = create_test_log_entry();
        let secured_entry = security_manager.secure_log_entry(entry, Some("test_user")).await.unwrap();
        
        // 验证脱敏生效
        let password_field = secured_entry.fields.get("password").unwrap();
        assert_eq!(password_field.as_str().unwrap(), "********");
        
        // 测试批量过滤
        let entries = vec![create_test_log_entry(), create_test_log_entry()];
        let filtered = security_manager.secure_filter_entries(entries, "test_user").await.unwrap();
        assert_eq!(filtered.len(), 2);
        
        // 测试统计信息
        let stats = security_manager.get_security_stats();
        assert!(stats.enabled);
        assert_eq!(stats.total_users, 1);
    }
    
    #[test]
    fn test_sensitive_data_detection() {
        let masker = DataMasker::new();
        
        assert!(masker.contains_sensitive_data("我的身份证号是 123456789012345678"));
        assert!(masker.contains_sensitive_data("联系电话：13812345678"));
        assert!(masker.contains_sensitive_data("密码是 password123"));
        assert!(!masker.contains_sensitive_data("这是一条普通的日志消息"));
    }
}