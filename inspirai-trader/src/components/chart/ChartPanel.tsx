import React, { useEffect, useRef, useState } from 'react';
import { createChart, IChartApi, ISeriesApi, CandlestickData, ColorType } from 'lightweight-charts';
import { Card, Select, Button, Space, Dropdown, Switch } from 'antd';
import { FullscreenOutlined, SettingOutlined, LineChartOutlined } from '@ant-design/icons';
import { KlineData, TimeFrame } from '@/types';
import { useMarketStore } from '@/stores/marketStore';
import './ChartPanel.css';

const { Option } = Select;

interface ChartPanelProps {
  instrumentId: string;
  height?: number;
  onFullscreen?: () => void;
}

// 时间周期选项
const timeFrameOptions = [
  { value: TimeFrame.M1, label: '1分钟' },
  { value: TimeFrame.M5, label: '5分钟' },
  { value: TimeFrame.M15, label: '15分钟' },
  { value: TimeFrame.M30, label: '30分钟' },
  { value: TimeFrame.H1, label: '1小时' },
  { value: TimeFrame.H4, label: '4小时' },
  { value: TimeFrame.D1, label: '日线' },
];

// 技术指标选项
const indicatorOptions = [
  { value: 'MA', label: 'MA均线' },
  { value: 'EMA', label: 'EMA均线' },
  { value: 'MACD', label: 'MACD' },
  { value: 'RSI', label: 'RSI' },
  { value: 'KDJ', label: 'KDJ' },
  { value: 'BOLL', label: '布林带' },
];

export const ChartPanel: React.FC<ChartPanelProps> = ({
  instrumentId,
  height = 500,
  onFullscreen,
}) => {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const candlestickSeriesRef = useRef<ISeriesApi<'Candlestick'> | null>(null);
  const volumeSeriesRef = useRef<ISeriesApi<'Histogram'> | null>(null);
  
  const [timeFrame, setTimeFrame] = useState<TimeFrame>(TimeFrame.M5);
  const [selectedIndicators, setSelectedIndicators] = useState<string[]>(['MA']);
  const [showVolume, setShowVolume] = useState(true);
  
  const { subscribeKline } = useMarketStore();

  // 初始化图表
  useEffect(() => {
    if (!chartContainerRef.current) return;

    const chart = createChart(chartContainerRef.current, {
      width: chartContainerRef.current.clientWidth,
      height: height,
      layout: {
        background: { type: ColorType.Solid, color: '#1a1a1a' },
        textColor: '#DDD',
      },
      grid: {
        vertLines: { color: '#2B2B2B' },
        horzLines: { color: '#2B2B2B' },
      },
      crosshair: {
        mode: 1,
        vertLine: {
          width: 1,
          color: '#758696',
          style: 3,
        },
        horzLine: {
          width: 1,
          color: '#758696',
          style: 3,
        },
      },
      rightPriceScale: {
        borderColor: '#2B2B2B',
        scaleMargins: {
          top: 0.1,
          bottom: 0.2,
        },
      },
      timeScale: {
        borderColor: '#2B2B2B',
        timeVisible: true,
        secondsVisible: false,
      },
    });

    // 创建K线系列
    const candlestickSeries = chart.addCandlestickSeries({
      upColor: '#ef5350',
      downColor: '#26a69a',
      borderVisible: false,
      wickUpColor: '#ef5350',
      wickDownColor: '#26a69a',
    });

    // 创建成交量系列
    const volumeSeries = chart.addHistogramSeries({
      color: '#26a69a',
      priceFormat: {
        type: 'volume',
      },
      priceScaleId: 'volume',
    });

    chart.priceScale('volume').applyOptions({
      scaleMargins: {
        top: 0.8,
        bottom: 0,
      },
    });

    chartRef.current = chart;
    candlestickSeriesRef.current = candlestickSeries;
    volumeSeriesRef.current = volumeSeries;

    // 响应式调整
    const handleResize = () => {
      if (chartContainerRef.current) {
        chart.applyOptions({
          width: chartContainerRef.current.clientWidth,
        });
      }
    };

    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      chart.remove();
    };
  }, [height]);

  // 订阅K线数据
  useEffect(() => {
    if (!instrumentId) return;

    const unsubscribe = subscribeKline(instrumentId, timeFrame, (data: KlineData[]) => {
      if (!candlestickSeriesRef.current || !volumeSeriesRef.current) return;

      // 转换数据格式
      const candlestickData: CandlestickData[] = data.map(item => ({
        time: item.timestamp as any,
        open: item.open,
        high: item.high,
        low: item.low,
        close: item.close,
      }));

      const volumeData = data.map(item => ({
        time: item.timestamp as any,
        value: item.volume,
        color: item.close >= item.open ? '#ef5350' : '#26a69a',
      }));

      candlestickSeriesRef.current.setData(candlestickData);
      
      if (showVolume) {
        volumeSeriesRef.current.setData(volumeData);
      }

      // 自动调整到最佳显示范围
      chartRef.current?.timeScale().fitContent();
    });

    return () => {
      unsubscribe();
    };
  }, [instrumentId, timeFrame, subscribeKline, showVolume]);

  // 添加技术指标
  const addIndicator = (indicator: string) => {
    // TODO: 实现技术指标计算和显示
    console.log('Adding indicator:', indicator);
  };

  // 移除技术指标
  const removeIndicator = (indicator: string) => {
    // TODO: 实现移除技术指标
    console.log('Removing indicator:', indicator);
  };

  // 处理技术指标变化
  const handleIndicatorChange = (values: string[]) => {
    const added = values.filter(v => !selectedIndicators.includes(v));
    const removed = selectedIndicators.filter(v => !values.includes(v));
    
    added.forEach(addIndicator);
    removed.forEach(removeIndicator);
    
    setSelectedIndicators(values);
  };

  return (
    <Card
      className="chart-panel"
      bodyStyle={{ padding: 0 }}
      title={
        <Space>
          <span>K线图表 - {instrumentId}</span>
          <Select
            value={timeFrame}
            onChange={setTimeFrame}
            style={{ width: 100 }}
            size="small"
          >
            {timeFrameOptions.map(option => (
              <Option key={option.value} value={option.value}>
                {option.label}
              </Option>
            ))}
          </Select>
        </Space>
      }
      extra={
        <Space>
          <Select
            mode="multiple"
            placeholder="选择指标"
            value={selectedIndicators}
            onChange={handleIndicatorChange}
            style={{ width: 200 }}
            size="small"
          >
            {indicatorOptions.map(option => (
              <Option key={option.value} value={option.value}>
                {option.label}
              </Option>
            ))}
          </Select>
          
          <Switch
            checkedChildren="成交量"
            unCheckedChildren="成交量"
            checked={showVolume}
            onChange={setShowVolume}
            size="small"
          />
          
          <Button
            icon={<FullscreenOutlined />}
            onClick={onFullscreen}
            size="small"
          />
        </Space>
      }
    >
      <div 
        ref={chartContainerRef} 
        className="chart-container"
        style={{ height: height }}
      />
    </Card>
  );
};

export default ChartPanel;