import React, { useState } from 'react';
import { Modal, Input, List, Button, Space, message, Popconfirm, Tag } from 'antd';
import {
  PlusOutlined,
  DeleteOutlined,
  DragOutlined,
  BellOutlined,
  SettingOutlined,
} from '@ant-design/icons';
import { DragDropContext, Droppable, Draggable } from 'react-beautiful-dnd';
import { useMarketDataStore } from '@stores/marketData';

interface WatchlistManagerProps {
  visible: boolean;
  onClose: () => void;
}

/**
 * 自选合约管理器
 */
const WatchlistManager: React.FC<WatchlistManagerProps> = ({ visible, onClose }) => {
  const {
    watchlist,
    subscriptions,
    addToWatchlist,
    removeFromWatchlist,
    reorderWatchlist,
    subscribe,
    unsubscribe,
  } = useMarketDataStore();

  const [inputValue, setInputValue] = useState('');
  const [isAdding, setIsAdding] = useState(false);

  // 添加合约到自选
  const handleAdd = async () => {
    if (!inputValue.trim()) {
      message.warning('请输入合约代码');
      return;
    }

    const instrumentId = inputValue.trim().toUpperCase();
    
    if (watchlist.includes(instrumentId)) {
      message.warning('该合约已在自选列表中');
      return;
    }

    setIsAdding(true);
    try {
      addToWatchlist(instrumentId);
      await subscribe(instrumentId);
      setInputValue('');
      message.success(`已添加 ${instrumentId} 到自选列表`);
    } catch (error) {
      message.error('添加失败');
    } finally {
      setIsAdding(false);
    }
  };

  // 移除合约
  const handleRemove = async (instrumentId: string) => {
    try {
      removeFromWatchlist(instrumentId);
      await unsubscribe(instrumentId);
      message.success(`已移除 ${instrumentId}`);
    } catch (error) {
      message.error('移除失败');
    }
  };

  // 拖拽结束处理
  const handleDragEnd = (result: any) => {
    if (!result.destination || result.source.droppableId !== 'watchlist') return;

    const items = Array.from(watchlist);
    const [reorderedItem] = items.splice(result.source.index, 1);
    if (reorderedItem) {
      items.splice(result.destination.index, 0, reorderedItem);
    }

    reorderWatchlist(items);
  };

  // 设置价格预警
  const handleSetAlert = (_instrumentId: string) => {
    // TODO: 实现价格预警设置
    message.info('价格预警功能开发中...');
  };

  return (
    <Modal
      title="管理自选合约"
      open={visible}
      onCancel={onClose}
      footer={null}
      width={600}
    >
      <Space direction="vertical" style={{ width: '100%' }}>
        <Input.Search
          placeholder="输入合约代码（如 rb2501）"
          enterButton={
            <Button
              type="primary"
              icon={<PlusOutlined />}
              loading={isAdding}
            >
              添加
            </Button>
          }
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          onSearch={handleAdd}
        />

        <DragDropContext onDragEnd={handleDragEnd}>
          <Droppable droppableId="watchlist">
            {(provided) => (
              <List
                {...provided.droppableProps}
                ref={provided.innerRef}
                dataSource={watchlist}
                locale={{ emptyText: '暂无自选合约' }}
                renderItem={(instrumentId, index) => {
                  const subscription = subscriptions.get(instrumentId);
                  const isSubscribed = subscription?.subscribed || false;
                  
                  return (
                    <Draggable
                      key={instrumentId}
                      draggableId={instrumentId}
                      index={index}
                    >
                      {(provided, snapshot) => (
                        <List.Item
                          ref={provided.innerRef}
                          {...provided.draggableProps}
                          style={{
                            ...provided.draggableProps.style,
                            backgroundColor: snapshot.isDragging ? 'var(--hover-color)' : undefined,
                          }}
                          actions={[
                            <Button
                              key="alert"
                              type="text"
                              size="small"
                              icon={<BellOutlined />}
                              onClick={() => handleSetAlert(instrumentId)}
                            >
                              预警
                            </Button>,
                            <Popconfirm
                              key="delete"
                              title="确定要移除该合约吗？"
                              onConfirm={() => handleRemove(instrumentId)}
                              okText="确定"
                              cancelText="取消"
                            >
                              <Button
                                type="text"
                                danger
                                size="small"
                                icon={<DeleteOutlined />}
                              >
                                移除
                              </Button>
                            </Popconfirm>,
                          ]}
                        >
                          <List.Item.Meta
                            avatar={
                              <div {...provided.dragHandleProps}>
                                <DragOutlined style={{ cursor: 'grab' }} />
                              </div>
                            }
                            title={
                              <Space>
                                <span>{instrumentId}</span>
                                <Tag color={isSubscribed ? 'success' : 'default'}>
                                  {isSubscribed ? '已订阅' : '未订阅'}
                                </Tag>
                              </Space>
                            }
                            description={
                              subscription?.error ? (
                                <span style={{ color: 'var(--color-down)' }}>
                                  {subscription.error}
                                </span>
                              ) : (
                                `排序：${index + 1}`
                              )
                            }
                          />
                        </List.Item>
                      )}
                    </Draggable>
                  );
                }}
              />
            )}
          </Droppable>
        </DragDropContext>

        <div style={{ marginTop: 16, textAlign: 'center', color: 'var(--text-muted)' }}>
          <SettingOutlined /> 拖拽调整顺序 | 共 {watchlist.length} 个自选合约
        </div>
      </Space>
    </Modal>
  );
};

export default WatchlistManager;