import React, { useState, useCallback } from 'react';

interface ResizablePanelProps {
  children: React.ReactNode;
  minWidth?: number;
  minHeight?: number;
  defaultWidth?: number;
  defaultHeight?: number;
  onResize?: (width: number, height: number) => void;
  className?: string;
}

const ResizablePanel: React.FC<ResizablePanelProps> = ({
  children,
  minWidth = 200,
  minHeight = 150,
  defaultWidth = 400,
  defaultHeight = 300,
  onResize,
  className = ''
}) => {
  const [size, setSize] = useState({
    width: defaultWidth,
    height: defaultHeight
  });

  const [isResizing, setIsResizing] = useState(false);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
  }, []);

  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (!isResizing) return;

      const newWidth = Math.max(minWidth, e.clientX);
      const newHeight = Math.max(minHeight, e.clientY);

      setSize({ width: newWidth, height: newHeight });
      onResize?.(newWidth, newHeight);
    },
    [isResizing, minWidth, minHeight, onResize]
  );

  const handleMouseUp = useCallback(() => {
    setIsResizing(false);
  }, []);

  React.useEffect(() => {
    if (isResizing) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
      
      return () => {
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
      };
    }
    
    // 如果不在 resizing 状态，返回 undefined（可选）
    return undefined;
  }, [isResizing, handleMouseMove, handleMouseUp]);

  return (
    <div 
      className={`relative ${className}`}
      style={{ width: size.width, height: size.height }}
    >
      {children}
      <div
        className="absolute bottom-0 right-0 w-4 h-4 cursor-se-resize hover:bg-active-color/20"
        onMouseDown={handleMouseDown}
      >
        <svg
          className="w-full h-full text-text-muted"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
        >
          <path d="M21 21L9 9M21 21V15M21 21H15" strokeWidth="2" />
        </svg>
      </div>
    </div>
  );
};

export default ResizablePanel;