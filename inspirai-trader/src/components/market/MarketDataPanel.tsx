import React, { useMemo, useState, useCallback } from 'react';
import { Table, Input, Button, Space, Tag, Tooltip, Badge } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import {
  StarOutlined,
  StarFilled,
  SearchOutlined,
  ReloadOutlined,
  BellOutlined,
} from '@ant-design/icons';
import { useMarketDataStore } from '@stores/marketData';
import type { MarketDataTick } from '@/types';
import { formatPrice, formatNumber, formatPercent } from '@utils/format';
import './MarketDataPanel.css';

const { Search } = Input;

/**
 * 行情数据面板组件
 */
const MarketDataPanel: React.FC = () => {
  const {
    ticks,
    watchlist,
    subscriptions,
    connectionStatus,
    addToWatchlist,
    removeFromWatchlist,
    subscribe,
  } = useMarketDataStore();

  const [searchText, setSearchText] = useState('');
  const [sortField, setSortField] = useState<string>('');
  const [sortOrder, setSortOrder] = useState<'ascend' | 'descend'>('ascend');
  const [selectedContract, setSelectedContract] = useState<string | null>(null);

  // 转换 Map 为数组并过滤
  const marketData = useMemo(() => {
    const dataArray = Array.from(ticks.values());
    
    // 搜索过滤
    let filtered = dataArray;
    if (searchText) {
      filtered = dataArray.filter(item =>
        item.instrumentId.toLowerCase().includes(searchText.toLowerCase())
      );
    }

    // 排序
    if (sortField) {
      filtered.sort((a, b) => {
        const aValue = (a as any)[sortField];
        const bValue = (b as any)[sortField];
        
        if (typeof aValue === 'number' && typeof bValue === 'number') {
          return sortOrder === 'ascend' ? aValue - bValue : bValue - aValue;
        }
        return 0;
      });
    }

    return filtered;
  }, [ticks, searchText, sortField, sortOrder]);

  // 判断是否在自选列表中
  const isInWatchlist = useCallback((instrumentId: string) => {
    return watchlist.includes(instrumentId);
  }, [watchlist]);

  // 切换自选状态
  const toggleWatchlist = useCallback((instrumentId: string) => {
    if (isInWatchlist(instrumentId)) {
      removeFromWatchlist(instrumentId);
    } else {
      addToWatchlist(instrumentId);
      // 自动订阅加入自选的合约
      if (!subscriptions.get(instrumentId)?.subscribed) {
        subscribe(instrumentId);
      }
    }
  }, [isInWatchlist, addToWatchlist, removeFromWatchlist, subscribe, subscriptions]);

  // 获取价格变动样式
  const getPriceChangeClass = (change: number) => {
    if (change > 0) return 'price-up';
    if (change < 0) return 'price-down';
    return 'price-neutral';
  };

  // 表格列定义
  const columns: ColumnsType<MarketDataTick> = [
    {
      title: '',
      key: 'watchlist',
      width: 40,
      fixed: 'left',
      render: (_, record) => (
        <Button
          type="text"
          size="small"
          icon={isInWatchlist(record.instrumentId) ? 
            <StarFilled className="text-warning" /> : 
            <StarOutlined />
          }
          onClick={() => toggleWatchlist(record.instrumentId)}
        />
      ),
    },
    {
      title: '合约代码',
      dataIndex: 'instrumentId',
      key: 'instrumentId',
      width: 100,
      fixed: 'left',
      sorter: true,
      render: (text) => (
        <Space>
          <span className="font-medium">{text}</span>
          {subscriptions.get(text)?.subscribed && (
            <Badge status="success" />
          )}
        </Space>
      ),
    },
    {
      title: '最新价',
      dataIndex: 'lastPrice',
      key: 'lastPrice',
      width: 100,
      sorter: true,
      render: (price) => (
        <span className={`font-tabular font-bold`}>
          {formatPrice(price)}
        </span>
      ),
    },
    {
      title: '涨跌',
      dataIndex: 'priceChange',
      key: 'priceChange',
      width: 80,
      sorter: true,
      render: (change) => (
        <span className={`font-tabular ${getPriceChangeClass(change || 0)}`}>
          {change > 0 ? '+' : ''}{formatPrice(change || 0)}
        </span>
      ),
    },
    {
      title: '涨跌幅',
      dataIndex: 'priceChangePercent',
      key: 'priceChangePercent',
      width: 80,
      sorter: true,
      render: (percent) => (
        <span className={`font-tabular ${getPriceChangeClass(percent || 0)}`}>
          {percent > 0 ? '+' : ''}{formatPercent(percent || 0)}%
        </span>
      ),
    },
    {
      title: '成交量',
      dataIndex: 'volume',
      key: 'volume',
      width: 100,
      sorter: true,
      render: (volume) => (
        <span className="font-tabular">{formatNumber(volume)}</span>
      ),
    },
    {
      title: '持仓量',
      dataIndex: 'openInterest',
      key: 'openInterest',
      width: 100,
      sorter: true,
      render: (openInterest) => (
        <span className="font-tabular">{formatNumber(openInterest)}</span>
      ),
    },
    {
      title: '买价',
      dataIndex: 'bidPrice1',
      key: 'bidPrice1',
      width: 80,
      render: (price) => (
        <span className="font-tabular text-color-up">{formatPrice(price)}</span>
      ),
    },
    {
      title: '卖价',
      dataIndex: 'askPrice1',
      key: 'askPrice1',
      width: 80,
      render: (price) => (
        <span className="font-tabular text-color-down">{formatPrice(price)}</span>
      ),
    },
    {
      title: '最高',
      dataIndex: 'highPrice',
      key: 'highPrice',
      width: 80,
      render: (price) => (
        <span className="font-tabular">{formatPrice(price)}</span>
      ),
    },
    {
      title: '最低',
      dataIndex: 'lowPrice',
      key: 'lowPrice',
      width: 80,
      render: (price) => (
        <span className="font-tabular">{formatPrice(price)}</span>
      ),
    },
    {
      title: '操作',
      key: 'action',
      width: 120,
      fixed: 'right',
      render: (_, record) => (
        <Space size="small">
          <Tooltip title="设置预警">
            <Button 
              type="text" 
              size="small" 
              icon={<BellOutlined />}
              onClick={() => handleSetAlert(record.instrumentId)}
            />
          </Tooltip>
          <Button
            type="link"
            size="small"
            onClick={() => handleViewDetail(record.instrumentId)}
          >
            详情
          </Button>
        </Space>
      ),
    },
  ];

  // 处理表格变化
  const handleTableChange = useCallback((_pagination: any, _filters: any, sorter: any) => {
    if (sorter.field) {
      setSortField(sorter.field);
      setSortOrder(sorter.order || 'ascend');
    }
  }, []);

  // 查看详情
  const handleViewDetail = (instrumentId: string) => {
    setSelectedContract(instrumentId);
    // TODO: 打开详情弹窗或切换到图表
  };

  // 设置预警
  const handleSetAlert = (instrumentId: string) => {
    // TODO: 打开预警设置弹窗
    console.log('Setting alert for', instrumentId);
  };

  // 刷新数据
  const handleRefresh = () => {
    // 重新订阅所有自选合约
    watchlist.forEach(instrumentId => {
      subscribe(instrumentId);
    });
  };

  return (
    <div className="market-data-panel">
      <div className="market-data-header">
        <Space>
          <Search
            placeholder="搜索合约代码或名称"
            allowClear
            onSearch={setSearchText}
            onChange={(e) => setSearchText(e.target.value)}
            style={{ width: 200 }}
            prefix={<SearchOutlined />}
          />
          <Button 
            icon={<ReloadOutlined />} 
            onClick={handleRefresh}
          >
            刷新
          </Button>
          <Tag color={connectionStatus === 'CONNECTED' ? 'success' : 'error'}>
            {connectionStatus === 'CONNECTED' ? '已连接' : '未连接'}
          </Tag>
        </Space>
      </div>
      
      <Table
        columns={columns}
        dataSource={marketData}
        rowKey="instrumentId"
        size="small"
        pagination={{
          pageSize: 20,
          showSizeChanger: true,
          showQuickJumper: true,
          showTotal: (total) => `共 ${total} 个合约`,
        }}
        scroll={{ x: 1200, y: 'calc(100vh - 250px)' }}
        onChange={handleTableChange}
        rowClassName={(record) => 
          selectedContract === record.instrumentId ? 'selected-row' : ''
        }
        onRow={(record) => ({
          onClick: () => setSelectedContract(record.instrumentId),
          onDoubleClick: () => handleViewDetail(record.instrumentId),
        })}
      />
    </div>
  );
};

export default MarketDataPanel;