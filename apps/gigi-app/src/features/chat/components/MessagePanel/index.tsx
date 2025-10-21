import { useState, useRef, useEffect, memo } from 'react';
import MessageBubble from './MessageBubble';
import MessageActionCard from './MessageActionCard';
import { senders } from '../../../../data/senders';
import type { Message } from '../../../../models/db';
import ImageMessageBubble from '../ImageMessageBubble'; // 导入图片消息组件

// 使用 memo 包装消息项以避免不必要的重新渲染
const MessageItem = memo(({
  message,
  me,
  selectedMessages,
  startPressTimer,
  cancelPressTimer,
  handleLongPress,
  handleMessageClick
}: {
  message: Message;
  me: string;
  selectedMessages: number[];
  startPressTimer: (messageId: number) => void;
  cancelPressTimer: () => void;
  handleLongPress: (messageId: number) => void;
  handleMessageClick: (messageId: number) => void;
}) => {
  const sender = senders.find((user) => user.id === message.sender) || {
    id: message.sender,
    name: message.sender,
    avatar: () => null
  };

  const isMe = message.sender === me;
  const isSelected = selectedMessages.includes(message.id || 0);

  // 检查是否为图片消息
  const isImageMessage = message.content.startsWith('[image:');
  const imageId = isImageMessage ? message.content.slice(7, -1) : null;

  return (
    <div
      key={message.id}
      className={`mb-6 relative ${isSelected ? 'bg-blue-50 rounded-xl -m-2 p-2' : ''} ${isMe ? 'flex justify-end' : 'flex'}`}
      onClick={() => handleMessageClick(message.id || 0)}
      onMouseDown={() => startPressTimer(message.id || 0)}
      onMouseUp={cancelPressTimer}
      onMouseLeave={cancelPressTimer}
      onContextMenu={(e) => {
        e.preventDefault();
        handleLongPress(message.id || 0);
      }}
      onTouchStart={() => startPressTimer(message.id || 0)}
      onTouchEnd={cancelPressTimer}
      onTouchCancel={cancelPressTimer}
    >
      {isImageMessage && imageId ? (
        // 渲染图片消息
        <div className={`flex flex-col ${isMe ? 'items-end' : 'items-start'}`}>
          {!isMe && (
            <div className="text-sm mb-1">
              {sender.name}
            </div>
          )}
          <ImageMessageBubble imageId={imageId} />
          <div className={`text-xs opacity-50 mt-1 ${isMe ? 'text-right' : ''}`}>
            {new Date(message.timestamp).toLocaleTimeString([], {
              hour: '2-digit',
              minute: '2-digit',
            })}
          </div>
        </div>
      ) : (
        // 渲染普通文本消息
        <MessageBubble
          message={message}
          sender={sender}
          isMe={isMe}
        />
      )}
    </div>
  );
});

MessageItem.displayName = 'MessageItem';

interface MessagePanelProps {
  messages: Message[];
  me: string;
  onMessageAction: (action: string, messageId: number) => void;
}

const MessagePanel = ({
  messages,
  me,
  onMessageAction,
}: MessagePanelProps) => {
  const [selectedMessageId, setSelectedMessageId] = useState<number | null>(
    null,
  );
  const [isMultiSelect, setIsMultiSelect] = useState(false);
  const [selectedMessages, setSelectedMessages] = useState<number[]>([]);
  const [actionCardPosition, setActionCardPosition] = useState({
    top: 0,
    left: 0,
  });
  const panelRef = useRef<HTMLDivElement>(null);

  // 修复长按问题
  const pressTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isLongPress = useRef(false);

  // 滚动到底部
  const scrollToBottom = () => {
    if (panelRef.current) {
      panelRef.current.scrollTop = panelRef.current.scrollHeight;
    }
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  // 修复长按功能 - 不再使用 contextmenu 和直接 preventDefault
  const startPressTimer = (messageId: number) => {
    isLongPress.current = false;

    // 设置1秒后触发长按事件
    pressTimer.current = setTimeout(() => {
      isLongPress.current = true;
      handleLongPress(messageId);
    }, 600); // 微信长按时间约为600ms
  };

  const cancelPressTimer = () => {
    if (pressTimer.current) {
      clearTimeout(pressTimer.current);
      pressTimer.current = null;
    }
  };

  const handleLongPress = (messageId: number) => {
    // 取消可能的定时器
    cancelPressTimer();

    setSelectedMessageId(messageId);

    if (panelRef.current) {
      const panelRect = panelRef.current.getBoundingClientRect();

      // 在消息面板中心位置显示动作卡片
      setActionCardPosition({
        top: panelRect.height / 2,
        left: panelRect.width / 2,
      });
    }
  };

  // 点击消息处理
  const handleMessageClick = (messageId: number) => {
    // 取消可能的定时器
    cancelPressTimer();

    // 如果长按已经触发，则忽略点击
    if (isLongPress.current) {
      isLongPress.current = false;
      return;
    }

    // 如果是多选模式，处理多选
    if (isMultiSelect) {
      handleMultiSelect(messageId);
    }
  };

  // 多选处理
  const handleMultiSelect = (messageId: number) => {
    if (selectedMessages.includes(messageId)) {
      setSelectedMessages((prev) => prev.filter((id) => id !== messageId));
    } else {
      setSelectedMessages((prev) => [...prev, messageId]);
    }
  };

  // 动作卡片操作处理
  const handleAction = (action: string, messageId: number) => {
    onMessageAction(action, messageId);
    setSelectedMessageId(null);

    // 处理多选操作
    if (action === '多选') {
      setIsMultiSelect(true);
      setSelectedMessages([messageId]);
    }

    // 处理退出多选模式
    if (action === '删除' && isMultiSelect) {
      setIsMultiSelect(false);
      setSelectedMessages([]);
    }
  };

  // 关闭所有操作卡片
  const closeActionCard = () => {
    setSelectedMessageId(null);
  };

  // 点击其他地方关闭动作卡片
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      const target = e.target as Element;
      if (selectedMessageId && (!target.closest || !target.closest('.message-action-card'))) {
        closeActionCard();
      }
    };

    const handleContextMenu = (e: MouseEvent) => {
      if (selectedMessageId) {
        e.preventDefault();
      }
    };

    if (selectedMessageId) {
      document.addEventListener('mousedown', handleClickOutside);
      document.addEventListener('contextmenu', handleContextMenu);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('contextmenu', handleContextMenu);
    };
  }, [selectedMessageId]);

  // 按ESC键关闭动作卡片
  useEffect(() => {
    const handleEscapeKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && selectedMessageId) {
        closeActionCard();
      }
    };

    if (selectedMessageId) {
      document.addEventListener('keydown', handleEscapeKey);
    }

    return () => {
      document.removeEventListener('keydown', handleEscapeKey);
    };
  }, [selectedMessageId]);

  // 清除所有定时器
  useEffect(() => {
    return () => {
      if (pressTimer.current) {
        clearTimeout(pressTimer.current);
      }
    };
  }, []);

  return (
    <div
      ref={panelRef}
      className="h-full p-4 pb-0"
      onContextMenu={(e) => {
        // 防止在面板上出现浏览器上下文菜单
        const target = e.target as Element;
        if (!target.closest || !target.closest('.message-action-card')) {
          e.preventDefault();
        }
      }}
    >
      {messages.map((message) => (
        <MessageItem
          key={message.id}
          message={message}
          me={me}
          selectedMessages={selectedMessages}
          startPressTimer={startPressTimer}
          cancelPressTimer={cancelPressTimer}
          handleLongPress={handleLongPress}
          handleMessageClick={handleMessageClick}
        />
      ))}

      {/* 动作卡片渲染 */}
      {selectedMessageId && (
        <MessageActionCard
          messageId={selectedMessageId}
          position={actionCardPosition}
          onAction={(action) => handleAction(action, selectedMessageId)}
          onClose={closeActionCard}
        />
      )}

      {/* 底部间距 */}
      <div className="h-12"></div>
    </div>
  );
};

export default MessagePanel;
