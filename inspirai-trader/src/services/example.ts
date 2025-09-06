/**
 * CTP 服务层使用示例
 * 
 * 演示如何使用 CTP 服务管理器
 */

import { ctpServiceManager } from './ctp.service';
import { getDefaultPreset } from '../config/ctp-presets';

/**
 * 基本使用示例
 */
export async function basicUsageExample() {
  try {
    // 1. 初始化服务
    console.log('初始化 CTP 服务...');
    await ctpServiceManager.init();
    
    // 2. 创建配置
    console.log('创建配置...');
    const config = ctpServiceManager.getDefaultConfig({
      userId: 'test_user',
      password: 'test_password',
    });
    console.log('配置创建成功:', config.environment);
    
    // 3. 获取服务状态
    console.log('获取服务配置...');
    const serviceConfig = ctpServiceManager.getServiceConfig();
    console.log('服务配置:', serviceConfig);
    
    console.log('CTP 服务层基本功能验证成功！');
    
  } catch (error) {
    console.error('CTP 服务层验证失败:', error);
  }
}

/**
 * 连接流程示例
 */
export async function connectionExample() {
  try {
    // 1. 从预设创建配置
    const config = ctpServiceManager.createConfigFromPreset('gzqh_test', {
      userId: 'demo_user',
      password: 'demo_password',
    });
    
    console.log('配置信息:');
    console.log('- 环境:', config.environment);
    console.log('- 经纪商:', config.broker_id);
    console.log('- 行情地址:', config.md_front_addr);
    console.log('- 交易地址:', config.trader_front_addr);
    
    // 2. 检查当前状态
    const currentConfig = ctpServiceManager.getCurrentConfig();
    console.log('当前配置:', currentConfig ? '已设置' : '未设置');
    
    console.log('连接流程示例完成！');
    
  } catch (error) {
    console.error('连接流程示例失败:', error);
  }
}

/**
 * 事件监听示例
 */
export async function eventListenerExample() {
  try {
    console.log('设置事件监听器...');
    
    // 监听行情数据（模拟）
    const unlistenMarketData = await ctpServiceManager.listenToMarketData((data) => {
      console.log('收到行情数据:', data.instrumentId, data.lastPrice);
    });
    
    // 监听连接状态（模拟）
    const unlistenConnection = await ctpServiceManager.listenToConnectionStatus((state) => {
      console.log('连接状态变化:', state);
    });
    
    // 监听错误事件（模拟）
    const unlistenErrors = await ctpServiceManager.listenToErrors((error) => {
      console.log('收到错误事件:', error.message);
    });
    
    console.log('事件监听器设置完成！');
    
    // 清理监听器
    setTimeout(async () => {
      console.log('清理事件监听器...');
      await ctpServiceManager.removeAllListeners();
      console.log('事件监听器已清理');
    }, 1000);
    
  } catch (error) {
    console.error('事件监听示例失败:', error);
  }
}

/**
 * 错误处理示例
 */
export async function errorHandlingExample() {
  try {
    console.log('测试错误处理...');
    
    // 测试无效预设
    try {
      ctpServiceManager.createConfigFromPreset('invalid_preset');
    } catch (error) {
      console.log('捕获到预期错误:', error.message);
    }
    
    // 测试错误分类
    const testError = ctpServiceManager.handleError(new Error('连接超时'), '测试上下文');
    console.log('错误分类结果:', testError.type);
    console.log('用户友好消息:', ctpServiceManager.getUserFriendlyMessage(testError));
    
    console.log('错误处理示例完成！');
    
  } catch (error) {
    console.error('错误处理示例失败:', error);
  }
}

// 如果直接运行此文件，执行示例
if (import.meta.main) {
  console.log('=== CTP 服务层示例 ===\n');
  
  await basicUsageExample();
  console.log('\n---\n');
  
  await connectionExample();
  console.log('\n---\n');
  
  await eventListenerExample();
  console.log('\n---\n');
  
  await errorHandlingExample();
  
  console.log('\n=== 示例完成 ===');
}