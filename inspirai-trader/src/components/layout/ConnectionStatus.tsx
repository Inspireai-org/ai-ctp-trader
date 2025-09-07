import React, { useState } from 'react';
import { Button, Badge, Tooltip, Space, Typography } from 'antd';
import { 
  LinkOutlined, 
  DisconnectOutlined, 
  UserOutlined,
  ReloadOutlined 
} from '@ant-design/icons';
import { useConnectionStore } from '@/stores/connectionStore';
import ConnectionDialog from '@/components/connection/ConnectionDialog';
import './ConnectionStatus.css';

const { Text } = Typography;

export const ConnectionStatus: React.FC = () => {
  const [showDialog, setShowDialog] = useState(false);
  
  const { 
    connectionState, 
    isConnected, 
    isLoggedIn, 
    userId,
    disconnect,
    connect
  } = useConnectionStore();

  const getStatusColor = () => {
    switch (connectionState) {
      case 'LoggedIn': return 'success';
      case 'Connected': return 'processing';
      case 'Connecting':
      case 'LoggingIn': return 'warning';
      default: return 'error';
    }
  };

  const getStatusText = () => {
    switch (connectionState) {
      case 'LoggedIn': return '已连接';
      case 'Connected': return '已连接(未登录)';
      case 'Connecting': return '连接中...';
      case 'LoggingIn': return '登录中...';
      case 'Disconnecting': return '断开中...';
      default: return '未连接';
    }
  };

  const handleReconnect = async () => {
    try {
      await connect();
    } catch (error) {
      console.error('Reconnect failed:', error);
    }
  };

  return (
    <>
      <div className="connection-status">
        <Space>
          <Badge status={getStatusColor() as any} />
          <Text className="status-text">{getStatusText()}</Text>
          
          {isLoggedIn && userId && (
            <Space>
              <UserOutlined />
              <Text className="user-id">{userId}</Text>
            </Space>
          )}
          
          {!isConnected ? (
            <Button 
              type="primary" 
              size="small"
              icon={<LinkOutlined />}
              onClick={() => setShowDialog(true)}
            >
              连接
            </Button>
          ) : (
            <Space>
              {!isLoggedIn && (
                <Tooltip title="重新连接">
                  <Button 
                    size="small"
                    icon={<ReloadOutlined />}
                    onClick={handleReconnect}
                  />
                </Tooltip>
              )}
              <Tooltip title="断开连接">
                <Button 
                  size="small"
                  danger
                  icon={<DisconnectOutlined />}
                  onClick={disconnect}
                />
              </Tooltip>
            </Space>
          )}
        </Space>
      </div>
      
      <ConnectionDialog 
        visible={showDialog}
        onClose={() => setShowDialog(false)}
      />
    </>
  );
};

export default ConnectionStatus;