import React, { Component, ErrorInfo, ReactNode } from 'react';
import { Result, Button, Typography, Card } from 'antd';
import { ReloadOutlined, BugOutlined } from '@ant-design/icons';

const { Paragraph, Text } = Typography;

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
  errorInfo?: ErrorInfo;
  errorId: string;
}

/**
 * 全局错误边界组件
 * 
 * 功能特性：
 * - 捕获 React 组件树中的 JavaScript 错误
 * - 显示友好的错误界面
 * - 提供错误详情和恢复选项
 * - 记录错误信息用于调试
 */
export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    
    this.state = {
      hasError: false,
      errorId: this.generateErrorId(),
    };
  }

  /**
   * 生成错误 ID
   */
  private generateErrorId(): string {
    return `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * 从错误中派生状态
   */
  static getDerivedStateFromError(error: Error): Partial<State> {
    return {
      hasError: true,
      error,
      errorId: `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
    };
  }

  /**
   * 捕获错误信息
   */
  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    this.setState({
      error,
      errorInfo,
    });

    // 记录错误到控制台
    console.error('ErrorBoundary 捕获到错误:', error);
    console.error('错误详情:', errorInfo);

    // 记录错误到本地存储（用于调试）
    this.logErrorToStorage(error, errorInfo);

    // 在开发环境下显示详细错误信息
    if (process.env.NODE_ENV === 'development') {
      console.group('🐛 错误边界详细信息');
      console.error('错误对象:', error);
      console.error('组件堆栈:', errorInfo.componentStack);
      console.error('错误堆栈:', error.stack);
      console.groupEnd();
    }
  }

  /**
   * 将错误记录到本地存储
   */
  private logErrorToStorage(error: Error, errorInfo: ErrorInfo) {
    try {
      const errorLog = {
        id: this.state.errorId,
        timestamp: new Date().toISOString(),
        message: error.message,
        stack: error.stack,
        componentStack: errorInfo.componentStack,
        userAgent: navigator.userAgent,
        url: window.location.href,
      };

      const existingLogs = JSON.parse(localStorage.getItem('error_logs') || '[]');
      existingLogs.push(errorLog);
      
      // 只保留最近的 10 条错误记录
      if (existingLogs.length > 10) {
        existingLogs.splice(0, existingLogs.length - 10);
      }
      
      localStorage.setItem('error_logs', JSON.stringify(existingLogs));
    } catch (storageError) {
      console.error('无法保存错误日志到本地存储:', storageError);
    }
  }

  /**
   * 重置错误状态
   */
  private handleReset = () => {
    this.setState({
      hasError: false,
      errorId: this.generateErrorId(),
    });
  };

  /**
   * 刷新页面
   */
  private handleReload = () => {
    window.location.reload();
  };

  /**
   * 复制错误信息
   */
  private handleCopyError = () => {
    const { error, errorInfo, errorId } = this.state;
    
    const errorText = `
错误 ID: ${errorId}
时间: ${new Date().toLocaleString()}
错误信息: ${error?.message || '未知错误'}
错误堆栈: ${error?.stack || '无堆栈信息'}
组件堆栈: ${errorInfo?.componentStack || '无组件堆栈'}
用户代理: ${navigator.userAgent}
页面地址: ${window.location.href}
    `.trim();

    navigator.clipboard.writeText(errorText).then(() => {
      console.log('错误信息已复制到剪贴板');
    }).catch((err) => {
      console.error('复制错误信息失败:', err);
    });
  };

  render() {
    if (this.state.hasError) {
      // 如果提供了自定义 fallback，使用它
      if (this.props.fallback) {
        return this.props.fallback;
      }

      const { error, errorId } = this.state;
      const isDevelopment = process.env.NODE_ENV === 'development';

      return (
        <div className="min-h-screen bg-bg-primary flex items-center justify-center p-4">
          <Card className="max-w-2xl w-full">
            <Result
              status="error"
              title="应用程序遇到错误"
              subTitle={`错误 ID: ${errorId}`}
              extra={[
                <Button 
                  type="primary" 
                  key="reset" 
                  icon={<ReloadOutlined />}
                  onClick={this.handleReset}
                >
                  重试
                </Button>,
                <Button 
                  key="reload" 
                  onClick={this.handleReload}
                >
                  刷新页面
                </Button>,
                isDevelopment && (
                  <Button 
                    key="copy" 
                    icon={<BugOutlined />}
                    onClick={this.handleCopyError}
                  >
                    复制错误信息
                  </Button>
                ),
              ].filter(Boolean)}
            >
              <div className="text-left">
                <Paragraph>
                  <Text strong>很抱歉，交易系统遇到了一个意外错误。</Text>
                </Paragraph>
                
                <Paragraph>
                  您可以尝试以下操作：
                </Paragraph>
                
                <ul className="list-disc list-inside space-y-1 text-text-secondary">
                  <li>点击"重试"按钮重新加载组件</li>
                  <li>点击"刷新页面"重新启动应用</li>
                  <li>检查网络连接是否正常</li>
                  <li>如果问题持续存在，请联系技术支持</li>
                </ul>

                {isDevelopment && error && (
                  <div className="mt-4 p-4 bg-bg-tertiary rounded border">
                    <Text strong className="text-color-down">开发模式 - 错误详情：</Text>
                    <pre className="mt-2 text-xs text-text-muted overflow-auto max-h-40">
                      {error.message}
                      {'\n\n'}
                      {error.stack}
                    </pre>
                  </div>
                )}
              </div>
            </Result>
          </Card>
        </div>
      );
    }

    return this.props.children;
  }
}

/**
 * 函数式错误边界 Hook（用于函数组件内部错误处理）
 */
export const useErrorHandler = () => {
  const handleError = React.useCallback((error: Error, errorInfo?: string) => {
    console.error('应用错误:', error);
    
    // 记录错误
    const errorLog = {
      id: `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      timestamp: new Date().toISOString(),
      message: error.message,
      stack: error.stack,
      info: errorInfo,
      userAgent: navigator.userAgent,
      url: window.location.href,
    };

    try {
      const existingLogs = JSON.parse(localStorage.getItem('error_logs') || '[]');
      existingLogs.push(errorLog);
      
      if (existingLogs.length > 10) {
        existingLogs.splice(0, existingLogs.length - 10);
      }
      
      localStorage.setItem('error_logs', JSON.stringify(existingLogs));
    } catch (storageError) {
      console.error('无法保存错误日志:', storageError);
    }
  }, []);

  return { handleError };
};

export default ErrorBoundary;