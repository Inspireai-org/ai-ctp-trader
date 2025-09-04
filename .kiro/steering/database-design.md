# 数据库设计规范

## 数据库表结构

### 用户账户表 (accounts)
```sql
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT UNIQUE NOT NULL,     -- 账户ID
    account_name TEXT NOT NULL,          -- 账户名称
    balance REAL NOT NULL DEFAULT 0,     -- 账户余额
    available_balance REAL NOT NULL DEFAULT 0, -- 可用资金
    frozen_balance REAL NOT NULL DEFAULT 0,    -- 冻结资金
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### 合约信息表 (contracts)
```sql
CREATE TABLE contracts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contract_code TEXT UNIQUE NOT NULL,  -- 合约代码
    contract_name TEXT NOT NULL,         -- 合约名称
    exchange TEXT NOT NULL,              -- 交易所
    product_type TEXT NOT NULL,          -- 产品类型
    trading_unit INTEGER NOT NULL,       -- 交易单位
    price_tick REAL NOT NULL,            -- 最小变动价位
    margin_rate REAL NOT NULL,           -- 保证金率
    is_active BOOLEAN DEFAULT 1,         -- 是否活跃
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### 订单表 (orders)
```sql
CREATE TABLE orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id TEXT UNIQUE NOT NULL,       -- 订单号
    account_id TEXT NOT NULL,            -- 账户ID
    contract_code TEXT NOT NULL,         -- 合约代码
    direction TEXT NOT NULL,             -- 买卖方向 (BUY/SELL)
    offset_flag TEXT NOT NULL,           -- 开平仓标志 (OPEN/CLOSE)
    price REAL NOT NULL,                 -- 委托价格
    volume INTEGER NOT NULL,             -- 委托数量
    traded_volume INTEGER DEFAULT 0,     -- 成交数量
    status TEXT NOT NULL,                -- 订单状态
    order_time DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(account_id),
    FOREIGN KEY (contract_code) REFERENCES contracts(contract_code)
);
```

### 持仓表 (positions)
```sql
CREATE TABLE positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,            -- 账户ID
    contract_code TEXT NOT NULL,         -- 合约代码
    direction TEXT NOT NULL,             -- 持仓方向 (LONG/SHORT)
    volume INTEGER NOT NULL,             -- 持仓量
    open_price REAL NOT NULL,            -- 开仓价格
    position_cost REAL NOT NULL,         -- 持仓成本
    margin REAL NOT NULL,                -- 占用保证金
    unrealized_pnl REAL DEFAULT 0,       -- 浮动盈亏
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(account_id),
    FOREIGN KEY (contract_code) REFERENCES contracts(contract_code)
);
```

## 索引设计
- 为经常查询的字段创建索引
- 复合索引优化多条件查询
- 定期分析查询性能并优化

## 数据迁移
- 使用版本化的数据库迁移脚本
- 每次结构变更都要有对应的迁移文件
- 支持数据库版本回滚机制