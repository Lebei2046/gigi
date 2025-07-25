/** biome-ignore-all lint/a11y/useKeyWithClickEvents: <explanation> */
import { useState, useRef, useEffect } from 'react';
import MessageBubble from './MessageBubble';
import MessageActionCard from './MessageActionCard';
import type { User } from '../../types';
import { contacts } from '../../../../data/contacts';
import type { Message } from '../../../../models/db';

interface MessagePanelProps {
  messages: Message[];
  currentUserId: string;
  onMessageAction: (action: string, messageId: number) => void;
}

const MessagePanel = ({
  messages,
  currentUserId,
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

  // 获取用户信息
  const getUserById = (id: string): User | undefined => {
    // First check in contacts data
    const contact = contacts.find((user) => user.id === id);
    if (contact) {
      return contact;
    }

    // If not found, return a default user object
    return {
      id: id,
      name: id,
      avatar: () => null
    };
  };

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
      const target = e.target as HTMLElement;
      if (selectedMessageId && !target.closest('.message-action-card')) {
        closeActionCard();
      }
    };

    document.addEventListener('click', handleClickOutside);
    return () => document.removeEventListener('click', handleClickOutside);
  }, [selectedMessageId]);

  // 按ESC键关闭动作卡片
  useEffect(() => {
    const handleEscapeKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && selectedMessageId) {
        closeActionCard();
      }
    };

    document.addEventListener('keydown', handleEscapeKey);
    return () => document.removeEventListener('keydown', handleEscapeKey);
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
      className="overflow-y-auto flex-grow p-4 pb-0"
      style={{ scrollBehavior: 'smooth' }}
    >
      {messages.length === 0
        ? <p>暂无数据</p>
        : messages.map((message) => {
          const sender = getUserById(message.sender);
          // Even if sender is not found, we still render the message
          const isCurrentUser = message.sender === currentUserId;
          const isSelected = selectedMessages.includes(message.id || 0);

          return (
            <div
              key={message.id}
              className={`mb-6 relative ${isSelected ? 'bg-blue-50 rounded-xl -m-2 p-2' : ''}`}
              onClick={() => handleMessageClick(message.id || 0)}
              onMouseDown={() => startPressTimer(message.id || 0)}
              onMouseUp={cancelPressTimer}
              onMouseLeave={cancelPressTimer}
              onTouchStart={() => startPressTimer(message.id || 0)}
              onTouchEnd={cancelPressTimer}
              onTouchCancel={cancelPressTimer}
            >
              <MessageBubble
                message={message}
                sender={sender || { id: message.sender, name: message.sender, avatar: () => null }}
                isCurrentUser={isCurrentUser}
              />
            </div>
          );
        })}

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
