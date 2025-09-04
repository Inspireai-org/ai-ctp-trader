# 期货交易界面需求文档

## 介绍

本文档定义了基于现有 inspirai-trader 项目的专业期货交易界面系统需求。项目已具备完整的 CTP 后端实现（基于 ctp2rs），包括行情服务、交易服务、账户服务等核心模块。本需求文档专注于前端界面的实现，将与已有的后端 Tauri 命令进行集成，提供完整的期货交易功能，包括实时行情展示、K线图表分析、交易下单、持仓管理等核心功能。界面设计参考专业期货交易软件的布局风格，采用深色主题，提供高效的交易体验。

## 项目现状

✅ **已完成的后端功能**：
- 完整的 CTP 客户端实现（CtpClient）
- 数据模型定义（MarketDataTick, OrderRequest, Position, AccountInfo 等）
- 事件驱动架构（CtpEvent, EventHandler）
- 查询服务（QueryService）、交易服务（TradingService）、账户服务（AccountService）
- 配置管理（ConfigManager）和错误处理（CtpError）
- 多环境支持（SimNow、TTS、生产环境）

✅ **已配置的技术栈**：
- React 19 + TypeScript + Tauri2
- Ant Design UI 组件库
- Zustand 状态管理
- ECharts 图表库
- Bun 包管理器

## 需求

### 需求 1：主界面布局设计

**用户故事：** 作为期货交易员，我希望有一个专业的多面板交易界面，以便同时监控行情、分析图表和执行交易操作。

#### 验收标准

1. WHEN 用户打开应用 THEN 系统 SHALL 显示经典的四象限交易布局（行情区、图表区、交易区、信息区）
2. WHEN 用户调整面板大小 THEN 系统 SHALL 支持拖拽调整各面板的尺寸比例
3. WHEN 用户切换显示器分辨率 THEN 界面 SHALL 自动适应不同屏幕尺寸
4. WHEN 用户右键点击面板标题栏 THEN 系统 SHALL 提供面板显示/隐藏的上下文菜单
5. WHEN 用户保存布局设置 THEN 系统 SHALL 记住用户的个性化布局配置

### 需求 2：实时行情展示模块

**用户故事：** 作为期货交易员，我需要实时查看多个合约的行情数据，以便快速了解市场动态和价格变化。

#### 验收标准

1. WHEN 系统接收到行情数据 THEN 界面 SHALL 在100ms内更新显示最新价格
2. WHEN 价格上涨 THEN 系统 SHALL 使用红色显示涨幅和相关数据
3. WHEN 价格下跌 THEN 系统 SHALL 使用绿色显示跌幅和相关数据
4. WHEN 用户点击合约代码 THEN 系统 SHALL 切换到该合约的详细行情
5. WHEN 行情数据异常或延迟 THEN 系统 SHALL 显示数据状态指示器
6. WHEN 用户添加自选合约 THEN 系统 SHALL 支持自定义行情列表
7. WHEN 用户设置价格预警 THEN 系统 SHALL 在达到预警价位时发出通知

### 需求 3：K线图表分析功能

**用户故事：** 作为期货交易员，我需要专业的K线图表工具来分析价格走势和技术指标，以便做出交易决策。

#### 验收标准

1. WHEN 用户选择合约 THEN 系统 SHALL 显示该合约的实时K线图
2. WHEN 用户切换时间周期 THEN 图表 SHALL 支持1分钟、5分钟、15分钟、30分钟、1小时、日线等周期
3. WHEN 用户添加技术指标 THEN 系统 SHALL 支持MA、MACD、RSI、KDJ、BOLL等常用指标
4. WHEN 用户在图表上绘图 THEN 系统 SHALL 支持趋势线、水平线、矩形等绘图工具
5. WHEN 用户缩放图表 THEN 系统 SHALL 支持鼠标滚轮缩放和拖拽平移
6. WHEN 用户双击图表 THEN 系统 SHALL 自动调整到最佳显示范围
7. WHEN 图表数据更新 THEN 系统 SHALL 平滑更新K线而不闪烁

### 需求 4：交易下单功能

**用户故事：** 作为期货交易员，我需要快速便捷的下单界面，以便及时执行交易策略。

#### 验收标准

1. WHEN 用户选择合约 THEN 下单面板 SHALL 自动填充合约信息和当前价格
2. WHEN 用户输入下单参数 THEN 系统 SHALL 实时计算保证金和手续费
3. WHEN 用户点击买入/卖出按钮 THEN 系统 SHALL 弹出确认对话框
4. WHEN 用户确认下单 THEN 系统 SHALL 在500ms内提交订单到CTP系统
5. WHEN 订单提交成功 THEN 系统 SHALL 显示订单编号和状态
6. WHEN 订单提交失败 THEN 系统 SHALL 显示详细的错误信息
7. WHEN 用户设置止损止盈 THEN 系统 SHALL 支持条件单功能
8. WHEN 用户快速下单 THEN 系统 SHALL 支持一键平仓和反手操作

### 需求 5：持仓管理界面

**用户故事：** 作为期货交易员，我需要清晰地查看和管理我的持仓情况，包括盈亏状态和风险控制。

#### 验收标准

1. WHEN 用户查看持仓 THEN 系统 SHALL 显示所有持仓合约的详细信息
2. WHEN 持仓盈亏变化 THEN 系统 SHALL 实时更新浮动盈亏和持仓市值
3. WHEN 用户右键点击持仓 THEN 系统 SHALL 提供平仓、加仓等快捷操作
4. WHEN 持仓达到预警线 THEN 系统 SHALL 发出风险提醒
5. WHEN 用户查看持仓详情 THEN 系统 SHALL 显示开仓时间、开仓价格、持仓天数等信息
6. WHEN 用户设置止损 THEN 系统 SHALL 支持移动止损和固定止损
7. WHEN 保证金不足 THEN 系统 SHALL 高亮显示风险持仓

### 需求 6：委托订单管理

**用户故事：** 作为期货交易员，我需要监控和管理我的所有委托订单，包括撤单和改单操作。

#### 验收标准

1. WHEN 用户查看委托 THEN 系统 SHALL 显示所有未成交订单的状态
2. WHEN 订单状态变化 THEN 系统 SHALL 实时更新订单信息
3. WHEN 用户撤销订单 THEN 系统 SHALL 在300ms内发送撤单指令
4. WHEN 用户批量撤单 THEN 系统 SHALL 支持一键撤销所有或指定条件的订单
5. WHEN 订单部分成交 THEN 系统 SHALL 显示已成交数量和剩余数量
6. WHEN 订单完全成交 THEN 系统 SHALL 自动从委托列表移除并记录到成交历史
7. WHEN 用户改单 THEN 系统 SHALL 支持修改价格和数量（如果交易所支持）

### 需求 7：账户资金监控

**用户故事：** 作为期货交易员，我需要实时监控账户资金状况，确保有足够的保证金进行交易。

#### 验收标准

1. WHEN 用户登录系统 THEN 界面 SHALL 显示账户总资产、可用资金、占用保证金
2. WHEN 资金发生变化 THEN 系统 SHALL 实时更新资金数据
3. WHEN 可用资金不足 THEN 系统 SHALL 显示红色警告提示
4. WHEN 用户查看资金详情 THEN 系统 SHALL 显示手续费、盈亏、出入金等明细
5. WHEN 风险度超过阈值 THEN 系统 SHALL 发出强制平仓预警
6. WHEN 用户设置资金预警 THEN 系统 SHALL 支持自定义预警阈值
7. WHEN 结算完成 THEN 系统 SHALL 更新账户权益和可用资金

### 需求 8：成交记录查询

**用户故事：** 作为期货交易员，我需要查询历史成交记录，以便分析交易表现和制定策略。

#### 验收标准

1. WHEN 用户查询成交记录 THEN 系统 SHALL 显示指定时间范围内的所有成交
2. WHEN 用户筛选记录 THEN 系统 SHALL 支持按合约、方向、日期等条件过滤
3. WHEN 用户导出数据 THEN 系统 SHALL 支持导出Excel或CSV格式
4. WHEN 用户查看成交详情 THEN 系统 SHALL 显示成交时间、价格、手续费等信息
5. WHEN 用户统计盈亏 THEN 系统 SHALL 计算指定期间的总盈亏和胜率
6. WHEN 数据量较大 THEN 系统 SHALL 支持分页加载和虚拟滚动
7. WHEN 用户搜索记录 THEN 系统 SHALL 支持模糊搜索和精确匹配

### 需求 9：界面主题和个性化

**用户故事：** 作为期货交易员，我希望能够自定义界面外观和操作习惯，以提高工作效率。

#### 验收标准

1. WHEN 用户切换主题 THEN 系统 SHALL 支持深色和浅色主题
2. WHEN 用户调整字体 THEN 系统 SHALL 支持字体大小和类型设置
3. WHEN 用户自定义颜色 THEN 系统 SHALL 允许修改涨跌颜色方案
4. WHEN 用户设置快捷键 THEN 系统 SHALL 支持自定义键盘快捷键
5. WHEN 用户保存设置 THEN 系统 SHALL 持久化所有个性化配置
6. WHEN 用户重置设置 THEN 系统 SHALL 提供恢复默认配置选项
7. WHEN 用户切换语言 THEN 系统 SHALL 支持中英文界面切换

### 需求 10：系统性能和稳定性

**用户故事：** 作为期货交易员，我需要系统具有高性能和稳定性，确保在关键时刻不会出现延迟或故障。

#### 验收标准

1. WHEN 系统处理大量行情数据 THEN 界面刷新率 SHALL 保持在60FPS以上
2. WHEN 内存使用超过阈值 THEN 系统 SHALL 自动清理缓存数据
3. WHEN 网络连接中断 THEN 系统 SHALL 自动重连并恢复数据订阅
4. WHEN 系统出现异常 THEN 应用 SHALL 记录错误日志并尝试恢复
5. WHEN 用户长时间使用 THEN 系统 SHALL 保持稳定运行不崩溃
6. WHEN 数据加载缓慢 THEN 界面 SHALL 显示加载进度指示器
7. WHEN 系统资源不足 THEN 应用 SHALL 降级显示以保证核心功能