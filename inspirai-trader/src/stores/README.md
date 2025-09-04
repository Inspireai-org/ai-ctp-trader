# 状态管理系统

本文档描述了期货交易界面的状态管理系统实现，基于 Zustand 构建，提供了完整的状态管理、持久化和事件通知功能。

## 架构概览

状态管理系统采用模块化设计，包含以下核心组件：

```
src/stores/
├── ui.ts              # UI 界面状态管理
├── marketData.ts      # 行情数据状态管理
├── trading.ts         # 交易状态管理
├── eventBus.ts        # 事件总线系统
├── stateManager.ts    # 状态管理器
├── initialize.ts      # 初始化和系统状态
└── index.ts          # 统一导出
```

## 核心功能

### 1. UI 状态管理 (ui.ts)

管理界面相关的所有状态，包括：

- **主题管理**: 深色/浅色主题切换，自定义颜色方案
- **布局管理**: 面板位置、大小、显示/隐藏状态
- **用户偏好**: 语言、数字格式、快捷键等个性化设置
- **界面状态**: 侧边栏折叠、全屏面板、活跃标签页

```typescript
import { useUIStore, useTheme, useLayout } from '@/stores';

// 使用主题
const { theme, switchTheme, isDark } = useTheme();

// 使用布局
const { layout, updateLayout, isLayoutLocked } = useLayout();
```

### 2. 行情数据状态管理 (marketData.ts)

管理实时行情数据和订阅状态：

- **连接管理**: 连接状态、心跳监控
- **行情数据**: 实时 Tick 数据、K线数据缓存
- **订阅管理**: 合约订阅、批量操作、状态跟踪
- **自选管理**: 自选合约列表、排序、持久化
- **合约信息**: 合约基础信息缓存

```typescript
import { useMarketData, useInstrumentTick } from '@/stores';

// 使用行情数据
const { subscribe, unsubscribe, getTick, watchlist } = useMarketData();

// 获取特定合约行情
const tick = useInstrumentTick('rb2501');
```

### 3. 交易状态管理 (trading.ts)

管理交易相关的所有状态：

- **账户管理**: 资金信息、风险度计算
- **订单管理**: 委托订单、状态跟踪、批量操作
- **持仓管理**: 持仓信息、盈亏计算、风险控制
- **成交记录**: 历史成交、统计分析
- **快速交易**: 一键下单、平仓、反手操作
- **风险控制**: 仓位限制、资金预警、风险评估

```typescript
import { useTrading, useInstrumentPositions } from '@/stores';

// 使用交易功能
const { 
  submitOrder, 
  cancelOrder, 
  quickBuy, 
  quickSell,
  accountInfo,
  checkRiskControl 
} = useTrading();

// 获取特定合约持仓
const positions = useInstrumentPositions('rb2501');
```

### 4. 事件总线系统 (eventBus.ts)

提供应用级事件通信机制：

- **事件发布订阅**: 类型安全的事件系统
- **事件历史**: 事件记录和查询
- **一次性订阅**: 自动取消的事件监听
- **错误处理**: 监听器异常隔离

```typescript
import { eventBus, useEventListener } from '@/stores';

// 监听事件
useEventListener(AppEventType.MARKET_DATA_UPDATED, (event) => {
  console.log('行情更新:', event.data);
});

// 发布事件
eventBus.emit(AppEventType.ORDER_UPDATED, orderData);
```

### 5. 状态管理器 (stateManager.ts)

提供状态备份、恢复和配置管理：

- **状态备份**: 创建、恢复、删除状态快照
- **配置导入导出**: JSON 格式的配置文件
- **自动备份**: 连接成功时自动备份
- **数据清理**: 定期清理过期数据

```typescript
import { useStateManager } from '@/stores';

const {
  createBackup,
  restoreBackup,
  exportConfig,
  importConfig,
  resetAllStates
} = useStateManager();
```

## 持久化策略

### 持久化内容

各个 Store 的持久化策略：

**UI Store**:
- ✅ 主题配置
- ✅ 布局设置
- ✅ 用户偏好
- ❌ 临时界面状态

**MarketData Store**:
- ✅ 自选合约列表
- ✅ 订阅配置（重启后需重新订阅）
- ✅ 合约信息缓存
- ❌ 实时行情数据
- ❌ K线数据

**Trading Store**:
- ✅ 快速交易配置
- ✅ 风险控制配置
- ✅ 交易统计信息
- ❌ 实时交易数据（需重新查询）

### 数据恢复

应用启动时的数据恢复流程：

1. **加载持久化配置**: 从 localStorage 恢复用户配置
2. **重置运行时状态**: 清空实时数据，重置连接状态
3. **重新构建 Map 对象**: 将序列化的数据重新构建为 Map
4. **初始化主题**: 应用主题设置到 DOM

## 事件系统

### 事件类型

系统定义了以下事件类型：

```typescript
enum AppEventType {
  // 连接事件
  CONNECTION_CHANGED = 'CONNECTION_CHANGED',
  
  // 市场数据事件
  MARKET_DATA_UPDATED = 'MARKET_DATA_UPDATED',
  KLINE_DATA_UPDATED = 'KLINE_DATA_UPDATED',
  
  // 交易事件
  ORDER_UPDATED = 'ORDER_UPDATED',
  POSITION_UPDATED = 'POSITION_UPDATED',
  TRADE_UPDATED = 'TRADE_UPDATED',
  ACCOUNT_UPDATED = 'ACCOUNT_UPDATED',
  
  // 系统事件
  ERROR_OCCURRED = 'ERROR_OCCURRED',
  NOTIFICATION = 'NOTIFICATION',
}
```

### 事件流

典型的事件流程：

1. **数据更新** → Store 更新状态 → 发布事件
2. **组件监听** → 接收事件 → 更新 UI
3. **跨模块通信** → 通过事件总线传递信息

## 初始化流程

### 系统初始化

```typescript
import { initializeStores } from '@/stores';

// 在应用启动时调用
await initializeStores();
```

初始化步骤：

1. **初始化状态管理器**: 设置事件监听和自动清理
2. **初始化 UI 主题**: 检测系统主题偏好
3. **设置全局错误处理**: 捕获未处理的错误
4. **设置状态同步监听器**: 监听关键状态变化
5. **创建初始化备份**: 保存初始状态

### 清理流程

```typescript
import { cleanupStores } from '@/stores';

// 在应用关闭时调用
cleanupStores();
```

## 性能优化

### 1. 数据结构优化

- 使用 `Map` 而非对象存储大量数据
- 避免深度嵌套的状态结构
- 合理使用 `subscribeWithSelector` 减少不必要的重渲染

### 2. 内存管理

- 定期清理过期的行情数据
- 限制事件历史记录大小
- 自动清理过期的备份

### 3. 事件优化

- 事件监听器异常隔离
- 防抖处理高频事件
- 批量处理相关事件

## 错误处理

### 1. 全局错误捕获

- 未捕获的 Promise 错误
- JavaScript 运行时错误
- 状态更新异常

### 2. 错误恢复

- 自动重连机制
- 状态回滚功能
- 降级显示策略

### 3. 错误通知

- 统一的错误事件
- 用户友好的错误提示
- 详细的错误日志

## 测试覆盖

测试文件：`src/stores/__tests__/stateManagement.test.ts`

测试覆盖：
- ✅ 状态初始化
- ✅ 状态更新操作
- ✅ 事件发布订阅
- ✅ 持久化功能
- ✅ 备份恢复
- ✅ 风险控制
- ✅ 配置导入导出

## 使用示例

### 完整的交易流程示例

```typescript
import { 
  useMarketData, 
  useTrading, 
  useEventListener,
  AppEventType 
} from '@/stores';

function TradingComponent() {
  const { subscribe, getTick } = useMarketData();
  const { submitOrder, checkRiskControl } = useTrading();
  
  // 监听行情更新
  useEventListener(AppEventType.MARKET_DATA_UPDATED, (event) => {
    const tick = event.data;
    console.log(`${tick.instrumentId} 最新价: ${tick.lastPrice}`);
  });
  
  // 订阅行情
  useEffect(() => {
    subscribe('rb2501');
  }, []);
  
  // 下单操作
  const handleOrder = async () => {
    const order = {
      instrumentId: 'rb2501',
      direction: OrderDirection.BUY,
      offsetFlag: OffsetFlag.OPEN,
      price: 3500,
      volume: 1,
      orderType: OrderType.LIMIT,
      timeCondition: TimeCondition.GFD,
    };
    
    // 风险控制检查
    const riskCheck = checkRiskControl(order);
    if (!riskCheck.allowed) {
      alert(riskCheck.reason);
      return;
    }
    
    try {
      const orderId = await submitOrder(order);
      console.log('订单提交成功:', orderId);
    } catch (error) {
      console.error('下单失败:', error);
    }
  };
  
  return (
    <div>
      <button onClick={handleOrder}>买入开仓</button>
    </div>
  );
}
```

## 最佳实践

### 1. 状态设计

- 保持状态结构扁平化
- 避免在状态中存储派生数据
- 使用合适的数据结构（Map vs Array vs Object）

### 2. 事件使用

- 使用类型安全的事件定义
- 避免事件循环依赖
- 及时清理事件监听器

### 3. 性能考虑

- 合理使用 React.memo 和 useMemo
- 避免在 render 中创建新对象
- 使用 subscribeWithSelector 精确订阅

### 4. 错误处理

- 始终处理异步操作的错误
- 提供用户友好的错误信息
- 记录详细的错误日志

## 扩展指南

### 添加新的 Store

1. 创建新的状态文件
2. 定义状态接口和默认值
3. 实现状态操作方法
4. 添加持久化配置（如需要）
5. 集成事件通知
6. 更新 index.ts 导出
7. 编写测试用例

### 添加新的事件类型

1. 在 types/index.ts 中定义事件类型
2. 在相应的 Store 中发布事件
3. 在需要的组件中监听事件
4. 更新事件处理逻辑

这个状态管理系统为期货交易界面提供了完整、可靠、高性能的状态管理解决方案，支持复杂的交易场景和用户需求。