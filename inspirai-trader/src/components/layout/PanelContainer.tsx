import React from 'react';
import { Card } from 'antd';

interface PanelContainerProps {
  title: string;
  className?: string;
  children: React.ReactNode;
  extra?: React.ReactNode;
}

const PanelContainer: React.FC<PanelContainerProps> = ({
  title,
  className = '',
  children,
  extra
}) => {
  return (
    <Card
      className={`h-full bg-bg-secondary border-border-color ${className}`}
      bodyStyle={{ 
        padding: 0, 
        height: 'calc(100% - 45px)',
        overflow: 'auto'
      }}
      title={
        <span className="text-text-primary text-sm font-medium">{title}</span>
      }
      extra={extra}
      bordered
    >
      {children}
    </Card>
  );
};

export default PanelContainer;