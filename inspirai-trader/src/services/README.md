# CTP 服务层文档

## 概述

CTP 服务层提供了完整的期货交易系统接口，包括连接管理、行情数据、交易功能、查询服务等。

## 主要组件

### CtpServiceManager

主要的服务管理器，采用单例模式设计。

```typescript
import { ctpServiceManager } from '@/services/ctp.service';

// 获取服务实例
const service = ctpServiceManager;
```

## 基本使用流程

### 1. 初始化服务

```typescript
// 初始化 CTP 服务
await ctpServiceManager.init();
```

### 2. 创建配置

```typescript
// 从预设创建配置
const config = ctpServiceManager.createConfigFromPreset('gzqh_test', {
  userId: 'your_user_id',
  password: 'your_password',
});

// 或者获取默认配置
const defaultConfig = ctpServiceManager.getDefaultConfig({
  userId: 'your_user_id',
  password: 'your_password',
});
```

### 3. 连接服务器

```typescript
// 普通连接
await ctpServiceManager.connect(config);

// 带重试的连接
await ctpServiceManager.connectWithRetry(config);
```

### 4. 用户登录

```typescript
const credentials = {
  brokerId: config.broker_id,
  userId: config.investor_id,
  password: config.password,
  appId: config.app_id,
  authCode: config.auth_code,
};

// 普通登录
const loginResponse = await ctpServiceManager.login(credentials);

// 带重试的登录
const loginResponse = await ctpServiceManager.loginWithRetry(credentials);
```

## 行情数据

### 订阅行情

```typescript
// 订阅单个合约
await ctpServiceManager.subscribeMarketData(['rb2510']);

// 订阅多个合约
await ctpServiceManager.subscribeMarketData(['rb2510', 'au2412', 'cu2501']);
```

### 取消订阅

```typescript
await ctpServiceManager.unsubscribeMarketData(['rb2510']);
```

### 监听行情数据

```typescript
const unlisten = await ctpServiceManager.listenToMarketData((data) => {
  console.log('收到行情:', data.instrumentId, data.lastPrice);
});

// 取消监听
unlisten();
```

## 交易功能

### 提交订单

```typescript
const orderRequest = {
  instrumentId: 'rb2510',
  direction: OrderDirection.BUY,
  offsetFlag: OffsetFlag.OPEN,
  price: 3600,
  volume: 1,
  orderType: OrderType.LIMIT,
  timeCondition: TimeCondition.GFD,
};

const orderId = await ctpServiceManager.submitOrder(orderRequest);
```

### 撤销订单

```typescript
await ctpServiceManager.cancelOrder(orderId);
```

## 查询功能

### 查询账户信息

```typescript
await ctpServiceManager.queryAccount();

// 监听账户信息更新
const unlisten = await ctpServiceManager.listenToAccountInfo((accountInfo) => {
  console.log('账户余额:', accountInfo.balance);
});
```

### 查询持仓信息

```typescript
await ctpServiceManager.queryPositions();

// 监听持仓信息更新
const unlisten = await ctpServiceManager.listenToPositionInfo((position) => {
  console.log('持仓:', position.instrumentId, position.totalPosition);
});
```

### 查询订单和成交

```typescript
// 查询所有订单
await ctpServiceManager.queryOrders();

// 查询指定合约的订单
await ctpServiceManager.queryOrders('rb2510');

// 查询成交记录
await ctpServiceManager.queryTrades();
```

## 状态管理

### 检查连接状态

```typescript
const isConnected = await ctpServiceManager.isConnected();
const isLoggedIn = await ctpServiceManager.isLoggedIn();
const state = await ctpServiceManager.getState();
```

### 监听连接状态变化

```typescript
const unlisten = await ctpServiceManager.listenToConnectionStatus((state) => {
  console.log('连接状态:', state);
});
```

## 错误处理

### 监听错误事件

```typescript
const unlisten = await ctpServiceManager.listenToErrors((error) => {
  console.error('CTP 错误:', error.message);
  
  // 检查错误类型
  if (ctpServiceManager.isConnectionError(error)) {
    console.log('这是连接错误');
  }
  
  // 获取用户友好的错误消息
  const friendlyMessage = ctpServiceManager.getUserFriendlyMessage(error);
  console.log('用户友好消息:', friendlyMessage);
});
```

### 错误分类

服务层会自动分类错误类型：

- `CONNECTION_ERROR`: 连接错误
- `AUTHENTICATION_ERROR`: 认证错误
- `CONFIG_ERROR`: 配置错误
- `TIMEOUT_ERROR`: 超时错误
- `NETWORK_ERROR`: 网络错误
- 等等...

## 配置管理

### 预设环境

系统提供了多个预设环境：

- `simnow`: SimNow 模拟环境
- `simnow_7x24`: SimNow 7x24 测试环境
- `openctp_sim`: OpenCTP 仿真环境
- `gzqh_test`: 广州期货评测环境
- `production_template`: 生产环境模板

### 服务配置

```typescript
// 获取服务配置
const config = ctpServiceManager.getServiceConfig();

// 更新服务配置
ctpServiceManager.updateServiceConfig({
  defaultRetry: {
    maxRetries: 5,
    baseDelay: 2000,
    exponentialBackoff: true,
  },
  connectionTimeout: 60000,
});
```

## 资源清理

### 清理事件监听器

```typescript
// 清理所有监听器
await ctpServiceManager.removeAllListeners();

// 清理指定监听器
await ctpServiceManager.removeListener('market-data');
```

### 断开连接和销毁

```typescript
// 断开连接
await ctpServiceManager.disconnect();

// 完全销毁服务实例
await ctpServiceManager.destroy();
```

## 完整示例

```typescript
import { ctpServiceManager } from '@/services/ctp.service';
import { OrderDirection, OffsetFlag, OrderType, TimeCondition } from '@/types';

async function tradingExample() {
  try {
    // 1. 初始化
    await ctpServiceManager.init();
    
    // 2. 创建配置
    const config = ctpServiceManager.createConfigFromPreset('gzqh_test', {
      userId: 'your_user_id',
      password: 'your_password',
    });
    
    // 3. 连接和登录
    await ctpServiceManager.connectWithRetry(config);
    
    const credentials = {
      brokerId: config.broker_id,
      userId: config.investor_id,
      password: config.password,
      appId: config.app_id,
      authCode: config.auth_code,
    };
    
    await ctpServiceManager.loginWithRetry(credentials);
    
    // 4. 设置事件监听
    await ctpServiceManager.listenToMarketData((data) => {
      console.log('行情更新:', data.instrumentId, data.lastPrice);
    });
    
    await ctpServiceManager.listenToOrderStatus((order) => {
      console.log('订单状态:', order.orderId, order.status);
    });
    
    // 5. 订阅行情
    await ctpServiceManager.subscribeMarketData(['rb2510']);
    
    // 6. 查询账户和持仓
    await ctpServiceManager.queryAccount();
    await ctpServiceManager.queryPositions();
    
    // 7. 下单
    const orderRequest = {
      instrumentId: 'rb2510',
      direction: OrderDirection.BUY,
      offsetFlag: OffsetFlag.OPEN,
      price: 3600,
      volume: 1,
      orderType: OrderType.LIMIT,
      timeCondition: TimeCondition.GFD,
    };
    
    const orderId = await ctpServiceManager.submitOrder(orderRequest);
    console.log('订单已提交:', orderId);
    
    // 8. 查询订单状态
    await ctpServiceManager.queryOrders();
    
  } catch (error) {
    console.error('交易示例失败:', error);
  }
}
```

## 注意事项

1. **环境要求**: 服务层需要在 Tauri 应用环境中运行，不能在纯 Node.js 环境中使用。

2. **错误处理**: 所有方法都会抛出 `CtpError` 类型的错误，建议使用 try-catch 进行处理。

3. **资源管理**: 记得在组件卸载时清理事件监听器，避免内存泄漏。

4. **重试机制**: 连接和登录方法提供了重试版本，建议在网络不稳定的环境中使用。

5. **状态检查**: 在执行交易操作前，建议先检查连接和登录状态。