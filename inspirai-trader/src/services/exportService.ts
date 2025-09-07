import { OrderStatus, Trade, Position, AccountInfo } from '@/types/ctp';

export class ExportService {
  // 导出为CSV
  static exportToCSV(data: any[], filename: string, headers?: string[]) {
    if (!data || data.length === 0) return;
    
    // 获取表头
    const csvHeaders = headers || Object.keys(data[0]);
    
    // 构建CSV内容
    let csvContent = csvHeaders.join(',') + '\n';
    
    data.forEach(row => {
      const values = csvHeaders.map(header => {
        const value = row[header as keyof typeof row];
        // 处理包含逗号的值
        if (typeof value === 'string' && value.includes(',')) {
          return `"${value}"`;
        }
        return value ?? '';
      });
      csvContent += values.join(',') + '\n';
    });
    
    // 创建Blob并下载
    const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8' });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(blob);
    link.download = `${filename}_${new Date().toISOString().slice(0, 10)}.csv`;
    link.click();
  }

  // 导出订单数据
  static exportOrders(orders: OrderStatus[]) {
    const data = orders.map(order => ({
      订单号: order.order_ref,
      合约: order.instrument_id,
      方向: order.direction === 'Buy' ? '买入' : '卖出',
      开平: order.offset,
      价格: order.price,
      数量: order.volume,
      已成交: order.volume_traded,
      剩余: order.volume_left,
      状态: this.getOrderStatusText(order.status),
      下单时间: order.insert_time,
      更新时间: order.update_time
    }));
    
    this.exportToCSV(data, 'orders');
  }

  // 导出成交记录
  static exportTrades(trades: Trade[]) {
    const data = trades.map(trade => ({
      成交号: trade.trade_id,
      订单号: trade.order_ref,
      合约: trade.instrument_id,
      方向: trade.direction === 'Buy' ? '买入' : '卖出',
      开平: trade.offset,
      成交价: trade.price,
      成交量: trade.volume,
      手续费: trade.commission,
      成交时间: trade.trade_time
    }));
    
    this.exportToCSV(data, 'trades');
  }

  // 导出持仓数据
  static exportPositions(positions: Position[]) {
    const data = positions.map(pos => ({
      合约: pos.instrument_id,
      方向: pos.direction === 'Buy' ? '多头' : '空头',
      总持仓: pos.position,
      今仓: pos.position_today,
      昨仓: pos.position_yesterday,
      均价: pos.average_price,
      持仓成本: pos.position_cost,
      保证金: pos.margin,
      平仓盈亏: pos.close_profit,
      持仓盈亏: pos.position_profit
    }));
    
    this.exportToCSV(data, 'positions');
  }

  // 导出账户资金
  static exportAccount(account: AccountInfo) {
    const data = [{
      账户ID: account.account_id,
      可用资金: account.available,
      账户余额: account.balance,
      占用保证金: account.margin,
      冻结保证金: account.frozen_margin,
      手续费: account.commission,
      平仓盈亏: account.close_profit,
      持仓盈亏: account.position_profit,
      风险度: `${account.risk_ratio}%`
    }];
    
    this.exportToCSV(data, 'account');
  }

  // 导出为JSON
  static exportToJSON(data: any, filename: string) {
    const json = JSON.stringify(data, null, 2);
    const blob = new Blob([json], { type: 'application/json' });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(blob);
    link.download = `${filename}_${new Date().toISOString().slice(0, 10)}.json`;
    link.click();
  }

  private static getOrderStatusText(status: string): string {
    const statusMap: Record<string, string> = {
      'Submitted': '已提交',
      'Accepted': '已接受',
      'Rejected': '已拒绝',
      'PartiallyFilled': '部分成交',
      'Filled': '全部成交',
      'Cancelled': '已撤销',
      'Cancelling': '撤销中'
    };
    return statusMap[status] || status;
  }
}