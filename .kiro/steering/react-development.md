# React 开发规范

## 组件设计原则
- 使用函数式组件和 React Hooks
- 组件职责单一，保持可复用性
- 使用 TypeScript 进行严格类型检查
- 组件命名使用 PascalCase，文件名与组件名保持一致

## 状态管理规范
- 全局状态使用 Zustand 管理（用户信息、行情数据、交易状态）
- 组件内部状态使用 useState
- 异步状态使用 useEffect + useState 或 React Query
- 避免 prop drilling，合理使用 Context

## 性能优化
- 使用 React.memo 包装纯组件
- 使用 useMemo 和 useCallback 优化重复计算
- 虚拟滚动处理大量数据列表
- 图表组件使用防抖处理频繁更新

## 代码风格
- 使用 ESLint + Prettier 统一代码格式
- 组件 props 使用 interface 定义类型
- 自定义 Hook 以 use 开头命名
- 事件处理函数以 handle 开头命名
- 使用 Bun 运行开发脚本和测试，享受更快的执行速度

## 开发工具配置
- 配置 `.bunfig.toml` 优化 Bun 性能
- 使用 Bun 的内置测试运行器进行单元测试
- 利用 Bun 的快速热重载提升开发体验

## 文件组织
```
src/
├── components/          # 通用组件
│   ├── Chart/          # 图表组件
│   ├── OrderForm/      # 下单表单
│   └── DataTable/      # 数据表格
├── pages/              # 页面组件
│   ├── Dashboard/      # 主面板
│   ├── Trading/        # 交易页面
│   └── History/        # 历史记录
├── hooks/              # 自定义 Hooks
├── stores/             # 状态管理
├── services/           # API 服务
└── types/              # 类型定义
```