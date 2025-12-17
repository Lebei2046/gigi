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

  // Add animation effect
  useEffect(() => {
    setVisible(true);
  }, []);

  // Handle action clicks
  const handleActionClick = (action: string) => {
    setVisible(false);
    setTimeout(() => {
      onAction(action);
    }, 300);
  };

  // Calculate actual card position - avoid going off screen
  const calculatePosition = useCallback(() => {
    const { innerWidth, innerHeight } = window;
    const cardWidth = 300;
    const cardHeight = 200;

    let left = position.left;
    let top = position.top;

    // Adjust horizontal position - avoid exceeding right edge
    if (left + cardWidth > innerWidth) {
      left = innerWidth - cardWidth - 20;
    }

    // Adjust vertical position - avoid exceeding bottom edge
    if (top + cardHeight > innerHeight) {
      top = innerHeight - cardHeight - 20;
    }

    return { top, left };
  }, [position]);

  const { top, left } = calculatePosition();
  const actions = [
    { name: 'Copy', icon: <FiCopy />, color: 'text-blue-500' },
    { name: 'Forward', icon: <FiShare2 />, color: 'text-green-500' },
    { name: 'Delete', icon: <FiTrash2 />, color: 'text-red-500' },
    { name: 'Multi-select', icon: <FiCheckSquare />, color: 'text-purple-500' },
    { name: 'Quote', icon: <FiMessageSquare />, color: 'text-yellow-500' },
    { name: 'Remind', icon: <FiBell />, color: 'text-orange-500' },
    { name: 'Translate', icon: <FiGlobe />, color: 'text-blue-400' },
    { name: 'Search', icon: <FiSearch />, color: 'text-teal-500' },
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
      {/* Title bar */}
      <div className="flex justify-between items-center mb-4 pb-2 border-b">
        <h3 className="text-sm font-medium text-gray-700">Message Actions</h3>
        <button
          type="button"
          onClick={onClose}
          className="p-1 rounded-full hover:bg-gray-100"
          aria-label="Close"
        >
          <FiX />
        </button>
      </div>

      {/* Action grid */}
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

      {/* Action bar in multi-select mode */}
      <div className="mt-4 pt-3 border-t">
        <button
          type="button"
          onClick={() => handleActionClick('Multi-select')}
          className="w-full py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
        >
          Select Messages
        </button>
      </div>
    </div>
  );
};

export default MessageActionCard;
