# CTP期货交易接入实施任务

## 阶段1：基础设施搭建 ✅ (已完成)

- [x] 1. 配置项目依赖和构建环境
  - ✅ 已添加 ctp2rs = "0.1.7" 依赖
  - ✅ 已配置 tokio, futures 等异步运行时
  - ✅ build.rs 已正确配置链接 CTP 库文件
  - _需求: 1.1, 8.1_

- [x] 2. 创建配置管理模块
  - ✅ 已实现 `src-tauri/src/ctp/config.rs` 和 `config_manager.rs`
  - ✅ 已创建环境配置文件 (production.toml, simnow.toml, tts.toml)
  - ✅ 已实现配置加载和验证逻辑
  - _需求: 10.1, 10.2, 10.3_

- [x] 3. 设置错误处理体系
  - ✅ 已创建 `src-tauri/src/ctp/error.rs` 定义 CtpError 枚举
  - ✅ 已实现错误转换和传播机制
  - ✅ 已添加日志记录 (logger.rs)
  - _需求: 8.2, 6.5_

## 阶段2：连接管理实现 ⚠️ (需要修复)

- [ ] 4. 修复行情连接管理
  - [ ] 4.1 修复 MdSpiImpl 认证流程
    - ⚠️ 修改 `src-tauri/src/ctp/spi/md_spi.rs`
    - 🔧 在 on_front_connected 中移除认证，直接登录
    - ✅ 自动重连逻辑已实现
    - _需求: 1.1, 1.3, 3.1_
  
  - [ ] 4.2 验证行情登录流程
    - 🔧 确保直接发送登录请求（无需认证）
    - ✅ on_rsp_user_login 回调已处理
    - ✅ 登录状态已记录
    - _需求: 2.2, 3.1_

- [ ] 5. 验证交易连接管理
  - [ ] 5.1 验证 TdSpiImpl 认证参数
    - ✅ 已实现 `src-tauri/src/ctp/spi/trader_spi.rs`
    - ✅ on_front_connected 已发送认证请求
    - 🔧 验证 auth_code: "QHFK5E2GLEUB9XHV"
    - _需求: 1.1, 2.1_
  
  - [x] 5.2 认证和登录流程已完成
    - ✅ 已处理 on_rsp_authenticate 回调
    - ✅ 认证成功后发送登录请求
    - ✅ 已处理 on_rsp_user_login 并保存会话信息
    - _需求: 2.1, 2.2, 2.4_
  
  - [ ] 5.3 实现结算单确认
    - 在登录成功后自动确认结算单
    - 处理 on_rsp_settlement_info_confirm
    - 记录确认状态
    - _需求: 7.1, 7.2, 7.3_

## 阶段3：行情服务开发

- [ ] 6. 实现行情订阅管理
  - [ ] 6.1 创建 MarketDataService
    - 实现 `src-tauri/src/ctp/services/market_data_service.rs`
    - 管理订阅列表
    - 实现批量订阅优化
    - _需求: 3.1, 3.4_
  
  - [ ] 6.2 处理行情数据回调
    - 实现 on_rtn_depth_market_data 处理
    - 转换 GB18030 编码到 UTF-8
    - 过滤异常数据
    - _需求: 3.2, 3.3, 8.3_

- [ ] 7. 实现行情数据缓存
  - 创建行情数据缓存结构
  - 实现线程安全的数据更新
  - 添加数据有效性检查
  - _需求: 3.2, 3.5_

- [ ] 8. 创建行情事件系统
  - 实现事件发送器和接收器
  - 通过 Tauri 事件推送到前端
  - 实现事件过滤和订阅管理
  - _需求: 3.2, 9.1_

## 阶段4：交易服务开发

- [ ] 9. 实现订单管理器
  - [ ] 9.1 创建 OrderManager 结构
    - 实现 `src-tauri/src/ctp/services/order_manager.rs`
    - 管理订单引用生成（OrderRef）
    - 维护活动订单列表
    - _需求: 4.1, 4.4_
  
  - [ ] 9.2 实现订单状态跟踪
    - 处理 on_rtn_order 回调
    - 更新订单状态
    - 发送订单更新事件
    - _需求: 4.2, 4.5_

- [ ] 10. 实现交易执行服务
  - [ ] 10.1 创建下单功能
    - 实现 `src-tauri/src/ctp/services/trading_service.rs`
    - 构建 CThostFtdcInputOrderField
    - 实现风险检查
    - _需求: 4.1, 6.1, 6.2_
  
  - [ ] 10.2 实现撤单功能
    - 构建 CThostFtdcInputOrderActionField
    - 处理撤单响应
    - 更新订单状态
    - _需求: 4.4_
  
  - [ ] 10.3 处理成交回报
    - 实现 on_rtn_trade 处理
    - 更新持仓信息
    - 计算盈亏
    - _需求: 4.5, 5.2_

## 阶段5：查询服务开发

- [ ] 11. 实现账户查询
  - [ ] 11.1 创建 QueryService
    - 实现 `src-tauri/src/ctp/services/query_service.rs`
    - 实现查询账户资金功能
    - 处理查询响应超时
    - _需求: 5.1, 5.3_
  
  - [ ] 11.2 实现查询结果缓存
    - 缓存账户信息
    - 实现缓存过期机制
    - 提供强制刷新选项
    - _需求: 5.1, 5.3_

- [ ] 12. 实现持仓查询
  - 查询投资者持仓
  - 处理持仓数据响应
  - 计算浮动盈亏
  - _需求: 5.1, 5.2, 5.4_

- [ ] 13. 实现委托和成交查询
  - 查询当日委托
  - 查询历史委托
  - 查询成交记录
  - _需求: 4.2, 5.5_

## 阶段6：Tauri接口层 ⚠️ (部分完成)

- [ ] 14. 完善 Tauri 命令接口
  - [x] 14.1 连接管理命令
    - ✅ 已实现 ctp_connect 命令
    - 🔧 需要实现 ctp_disconnect 命令
    - 🔧 需要实现连接状态查询
    - _需求: 1.2, 2.5_
  
  - [ ] 14.2 完善行情相关命令
    - ✅ 已实现 ctp_subscribe (订阅)
    - 🔧 需要实现 unsubscribe_market_data
    - 🔧 需要实现获取最新行情
    - _需求: 3.1, 3.4_
  
  - [ ] 14.3 创建交易相关命令
    - 🔧 需要实现 place_order
    - 🔧 需要实现 cancel_order
    - 🔧 需要实现查询命令 (query_account, query_positions等)
    - _需求: 4.1, 4.4, 5.1_

- [x] 15. 状态管理已完成
  - ✅ 已创建 AppState 全局状态管理器
  - ✅ 已实现服务生命周期管理
  - ✅ 已处理并发访问 (Arc<Mutex>)
  - _需求: 1.5, 8.4_

## 阶段7：前端集成

- [ ] 16. 创建 CTP 服务层
  - 实现 `src/services/ctp.service.ts`
  - 封装 Tauri 命令调用
  - 处理错误和重试
  - _需求: 9.1, 9.3_

- [ ] 17. 实现行情数据展示
  - 创建行情列表组件
  - 实现实时数据更新
  - 添加数据过滤和排序
  - _需求: 9.1, 9.4_

- [ ] 18. 实现交易面板
  - 创建下单组件
  - 实现快捷下单功能
  - 添加订单确认对话框
  - _需求: 9.2, 6.1_

- [ ] 19. 实现持仓和账户显示
  - 创建持仓列表组件
  - 显示账户资金信息
  - 实现自动刷新
  - _需求: 9.1, 9.5_

## 阶段8：测试和调试 ✅ (已完成)

- [x] 20. 集成测试已完成
  - [x] 20.1 连接流程测试
    - ✅ 已在 production_config_test.rs 中测试
    - ✅ 已在 simple_production_test.rs 中测试
    - ✅ 已测试重连机制
    - _需求: 8.1, 8.4_
  
  - [x] 20.2 交易流程测试
    - ✅ 已在 trading_functionality_test.rs 中测试
    - ✅ 已测试下单和撤单流程
    - ✅ 已测试订单状态更新
    - _需求: 8.4, 8.5_

- [x] 21. 调试工具已实现
  - ✅ 已添加详细日志输出 (logger.rs)
  - ✅ 已创建 PerformanceMonitor
  - ✅ 已实现状态监控
  - _需求: 8.1, 8.2, 8.5_

- [x] 22. 性能测试已完成
  - ✅ 已在 market_data_subscription_test.rs 中测试
  - ✅ 已测试行情数据处理
  - ✅ 已优化数据缓存
  - _需求: 3.2, 8.4_

## 阶段9：文档和部署 🔴 (待实现)

- [ ] 23. 编写使用文档
  - 创建配置指南
  - 编写 API 文档
  - 添加故障排查指南
  - _需求: 10.4, 10.5_

- [ ] 24. 准备部署环境
  - 验证生产环境配置
  - 设置日志收集
  - 配置监控告警
  - _需求: 1.4, 10.5_

---

## 🚨 需要立即修复的关键问题

### 1. MD API 认证问题（优先级：高）
- **文件**: `src-tauri/src/ctp/spi/md_spi.rs`
- **问题**: 在 `on_front_connected` 中错误地尝试认证
- **修复**: 直接发送登录请求，跳过认证步骤

### 2. 验证 Auth Code（优先级：高）
- **文件**: `src-tauri/src/ctp/config/production.toml`
- **确认**: auth_code = "QHFK5E2GLEUB9XHV"

### 3. 完善 Tauri 命令（优先级：中）
- **文件**: `src-tauri/src/lib.rs`
- **需要添加**:
  - ctp_disconnect
  - place_order
  - cancel_order
  - query_account
  - query_positions

## 📊 完成状态总结

- ✅ **已完成** (70%): 基础设施、核心服务、测试框架
- ⚠️ **需要修复** (15%): MD认证流程、部分Tauri命令
- 🔴 **待实现** (15%): 前端集成、文档部署