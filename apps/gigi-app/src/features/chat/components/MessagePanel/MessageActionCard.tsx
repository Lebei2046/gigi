import React, { useState, useEffect, useCallback } from 'react';
import {
  FiCopy,
  FiShare2,
  FiTrash2,
  FiCheckSquare,
  FiMessageSquare,
  FiBell,
  FiGlobe,
  FiSearch,
  FiX,
} from 'react-icons/fi';

interface MessageActionCardProps {
  messageId: number;
  position: { top: number; left: number };
  onAction: (action: string) => void;
  onClose: () => void;
}

const MessageActionCard: React.FC<MessageActionCardProps> = ({
  // messageId,
  position,
  onAction,
  onClose,
}) => {
  const [visible, setVisible] = useState(false);

  // 添加动画效果
  useEffect(() => {
    setVisible(true);
  }, []);

  // 处理动作点击
  const handleActionClick = (action: string) => {
    setVisible(false);
    setTimeout(() => {
      onAction(action);
    }, 300);
  };

  // 计算卡片的实际位置 - 避免超出屏幕
  const calculatePosition = useCallback(() => {
    const { innerWidth, innerHeight } = window;
    const cardWidth = 300;
    const cardHeight = 200;

    let left = position.left;
    let top = position.top;

    // 调整水平位置 - 避免超出右侧
    if (left + cardWidth > innerWidth) {
      left = innerWidth - cardWidth - 20;
    }

    // 调整垂直位置 - 避免超出底部
    if (top + cardHeight > innerHeight) {
      top = innerHeight - cardHeight - 20;
    }

    return { top, left };
  }, [position]);

  const { top, left } = calculatePosition();
  const actions = [
    { name: '复制', icon: <FiCopy />, color: 'text-blue-500' },
    { name: '转发', icon: <FiShare2 />, color: 'text-green-500' },
    { name: '删除', icon: <FiTrash2 />, color: 'text-red-500' },
    { name: '多选', icon: <FiCheckSquare />, color: 'text-purple-500' },
    { name: '引用', icon: <FiMessageSquare />, color: 'text-yellow-500' },
    { name: '提醒', icon: <FiBell />, color: 'text-orange-500' },
    { name: '翻译', icon: <FiGlobe />, color: 'text-blue-400' },
    { name: '搜一搜', icon: <FiSearch />, color: 'text-teal-500' },
  ];

  return (
    <div
      className={`
        fixed z-50 bg-white shadow-xl rounded-lg p-4 transition-all duration-300
        ${visible ? 'opacity-100 scale-100' : 'opacity-0 scale-95'}
      `}
      style={{
        top: `${top}px`,
        left: `${left}px`,
        transformOrigin: 'center center',
        width: '300px',
      }}
    >
      {/* 标题栏 */}
      <div className="flex justify-between items-center mb-4 pb-2 border-b">
        <h3 className="text-sm font-medium text-gray-700">消息操作</h3>
        <button
          type="button"
          onClick={onClose}
          className="p-1 rounded-full hover:bg-gray-100"
          aria-label="关闭"
        >
          <FiX />
        </button>
      </div>

      {/* 操作网格 */}
      <div className="grid grid-cols-4 gap-3">
        {actions.map((action) => (
          <button
            key={action.name}
            type="button"
            className={`flex flex-col items-center justify-center p-2 hover:bg-gray-100 rounded-lg transition-colors ${action.color}`}
            onClick={() => handleActionClick(action.name)}
            aria-label={action.name}
          >
            <div className="text-lg mb-1">{action.icon}</div>
            <span className="text-xs text-gray-700">{action.name}</span>
          </button>
        ))}
      </div>

      {/* 多选模式下的操作栏 */}
      <div className="mt-4 pt-3 border-t">
        <button
          type="button"
          onClick={() => handleActionClick('多选')}
          className="w-full py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
        >
          多选消息
        </button>
      </div>
    </div>
  );
};

export default MessageActionCard;
