/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // 主题色彩变量
        'bg-primary': 'var(--bg-primary)',
        'bg-secondary': 'var(--bg-secondary)',
        'bg-tertiary': 'var(--bg-tertiary)',
        'text-primary': 'var(--text-primary)',
        'text-secondary': 'var(--text-secondary)',
        'text-muted': 'var(--text-muted)',
        'color-up': 'var(--color-up)',
        'color-down': 'var(--color-down)',
        'color-neutral': 'var(--color-neutral)',
        'border-color': 'var(--border-color)',
        'hover-color': 'var(--hover-color)',
        'active-color': 'var(--active-color)',
        
        // 扩展的语义化颜色
        'success': 'var(--color-up)',
        'error': 'var(--color-down)',
        'warning': 'var(--color-neutral)',
        'info': 'var(--active-color)',
      },
      fontFamily: {
        'mono': [
          'SF Mono', 
          'Monaco', 
          'Inconsolata', 
          'Roboto Mono', 
          'Consolas',
          'Liberation Mono',
          'Menlo',
          'Courier',
          'monospace'
        ],
        'sans': [
          '-apple-system', 
          'BlinkMacSystemFont', 
          'Segoe UI', 
          'PingFang SC', 
          'Hiragino Sans GB', 
          'Microsoft YaHei', 
          'Helvetica Neue', 
          'Helvetica', 
          'Arial', 
          'sans-serif'
        ],
      },
      fontSize: {
        'xs': ['12px', { lineHeight: '16px' }],
        'sm': ['13px', { lineHeight: '18px' }],
        'base': ['14px', { lineHeight: '20px' }],
        'lg': ['16px', { lineHeight: '24px' }],
        'xl': ['18px', { lineHeight: '28px' }],
        '2xl': ['20px', { lineHeight: '32px' }],
      },
      spacing: {
        '18': '4.5rem',
        '88': '22rem',
        '128': '32rem',
      },
      borderRadius: {
        'sm': '2px',
        'DEFAULT': '4px',
        'md': '6px',
        'lg': '8px',
      },
      boxShadow: {
        'panel': '0 2px 8px rgba(0, 0, 0, 0.15)',
        'panel-hover': '0 4px 12px rgba(0, 0, 0, 0.25)',
        'inner-light': 'inset 0 1px 0 rgba(255, 255, 255, 0.1)',
      },
      animation: {
        'fade-in': 'fadeIn 0.2s ease-in-out',
        'slide-up': 'slideUp 0.3s ease-out',
        'slide-down': 'slideDown 0.3s ease-out',
        'pulse-slow': 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideUp: {
          '0%': { transform: 'translateY(10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        slideDown: {
          '0%': { transform: 'translateY(-10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
      },
      zIndex: {
        '60': '60',
        '70': '70',
        '80': '80',
        '90': '90',
        '100': '100',
      },
    },
  },
  plugins: [],
  // 重要：与 Ant Design 兼容性配置
  corePlugins: {
    preflight: false, // 禁用 Tailwind 的默认样式重置，避免与 Ant Design 冲突
  },
  // 确保在 Ant Design 组件中也能使用 Tailwind 类
  important: false,
}
