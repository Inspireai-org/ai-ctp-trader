import { useEffect, useState } from 'react';

/**
 * 响应式断点定义
 */
export const breakpoints = {
  xs: 480,   // 手机
  sm: 576,   // 大手机
  md: 768,   // 平板
  lg: 1024,  // 小型桌面
  xl: 1440,  // 标准桌面
  xxl: 1920, // 大屏幕
} as const;

/**
 * 响应式设备类型
 */
export type DeviceType = 'mobile' | 'tablet' | 'desktop' | 'widescreen';

/**
 * 响应式钩子返回值
 */
interface ResponsiveInfo {
  width: number;
  height: number;
  deviceType: DeviceType;
  isMobile: boolean;
  isTablet: boolean;
  isDesktop: boolean;
  isWidescreen: boolean;
  breakpoint: keyof typeof breakpoints;
}

/**
 * 响应式检测钩子
 */
export function useResponsive(): ResponsiveInfo {
  const [windowSize, setWindowSize] = useState({
    width: window.innerWidth,
    height: window.innerHeight,
  });

  useEffect(() => {
    let timeoutId: number;

    const handleResize = () => {
      clearTimeout(timeoutId);
      timeoutId = window.setTimeout(() => {
        setWindowSize({
          width: window.innerWidth,
          height: window.innerHeight,
        });
      }, 150); // 防抖
    };

    window.addEventListener('resize', handleResize);
    return () => {
      clearTimeout(timeoutId);
      window.removeEventListener('resize', handleResize);
    };
  }, []);

  // 计算当前断点
  const getBreakpoint = (width: number): keyof typeof breakpoints => {
    if (width < breakpoints.xs) return 'xs';
    if (width < breakpoints.sm) return 'sm';
    if (width < breakpoints.md) return 'md';
    if (width < breakpoints.lg) return 'lg';
    if (width < breakpoints.xl) return 'xl';
    return 'xxl';
  };

  // 计算设备类型
  const getDeviceType = (width: number): DeviceType => {
    if (width < breakpoints.md) return 'mobile';
    if (width < breakpoints.lg) return 'tablet';
    if (width < breakpoints.xxl) return 'desktop';
    return 'widescreen';
  };

  const breakpoint = getBreakpoint(windowSize.width);
  const deviceType = getDeviceType(windowSize.width);

  return {
    width: windowSize.width,
    height: windowSize.height,
    deviceType,
    isMobile: deviceType === 'mobile',
    isTablet: deviceType === 'tablet',
    isDesktop: deviceType === 'desktop',
    isWidescreen: deviceType === 'widescreen',
    breakpoint,
  };
}

/**
 * 获取响应式列数
 */
export function getResponsiveColumns(deviceType: DeviceType): number {
  switch (deviceType) {
    case 'mobile':
      return 6;
    case 'tablet':
      return 9;
    case 'desktop':
      return 12;
    case 'widescreen':
      return 12;
    default:
      return 12;
  }
}

/**
 * 获取响应式行高
 */
export function getResponsiveRowHeight(deviceType: DeviceType): number {
  switch (deviceType) {
    case 'mobile':
      return 40;
    case 'tablet':
      return 50;
    case 'desktop':
      return 60;
    case 'widescreen':
      return 60;
    default:
      return 60;
  }
}