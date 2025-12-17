import { useState, useRef, useEffect, memo } from 'react';
import MessageBubble from './MessageBubble';
import MessageActionCard from './MessageActionCard';
import { senders } from '../../../../data/senders';
import type { Message } from '../../../../models/db';
import ImageMessageBubble from '../ImageMessageBubble'; // Import image message component

// Wrap message item with memo to avoid unnecessary re-renders
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

  // Check if it's an image message
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
        // Render image message
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
        // Render regular text message
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

  // Fix long press issue
  const pressTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isLongPress = useRef(false);

  // Scroll to bottom
  const scrollToBottom = () => {
    if (panelRef.current) {
      panelRef.current.scrollTop = panelRef.current.scrollHeight;
    }
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  // Fix long press functionality - no longer using contextmenu and direct preventDefault
  const startPressTimer = (messageId: number) => {
    isLongPress.current = false;

    // Set long press event to trigger after 1 second
    pressTimer.current = setTimeout(() => {
      isLongPress.current = true;
      handleLongPress(messageId);
    }, 600); // WeChat long press time is about 600ms
  };

  const cancelPressTimer = () => {
    if (pressTimer.current) {
      clearTimeout(pressTimer.current);
      pressTimer.current = null;
    }
  };

  const handleLongPress = (messageId: number) => {
    // Cancel possible timer
    cancelPressTimer();

    setSelectedMessageId(messageId);

    if (panelRef.current) {
      const panelRect = panelRef.current.getBoundingClientRect();

      // Show action card at center of message panel
      setActionCardPosition({
        top: panelRect.height / 2,
        left: panelRect.width / 2,
      });
    }
  };

  // Handle message click
  const handleMessageClick = (messageId: number) => {
    // Cancel possible timer
    cancelPressTimer();

    // If long press already triggered, ignore click
    if (isLongPress.current) {
      isLongPress.current = false;
      return;
    }

    // If in multi-select mode, handle multi-selection
    if (isMultiSelect) {
      handleMultiSelect(messageId);
    }
  };

  // Handle multi-selection
  const handleMultiSelect = (messageId: number) => {
    if (selectedMessages.includes(messageId)) {
      setSelectedMessages((prev) => prev.filter((id) => id !== messageId));
    } else {
      setSelectedMessages((prev) => [...prev, messageId]);
    }
  };

  // Handle action card operations
  const handleAction = (action: string, messageId: number) => {
    onMessageAction(action, messageId);
    setSelectedMessageId(null);

    // Handle multi-select operation
    if (action === 'Multi-select') {
      setIsMultiSelect(true);
      setSelectedMessages([messageId]);
    }

    // Handle exit from multi-select mode
    if (action === 'Delete' && isMultiSelect) {
      setIsMultiSelect(false);
      setSelectedMessages([]);
    }
  };

  // Close all action cards
  const closeActionCard = () => {
    setSelectedMessageId(null);
  };

  // Click outside to close action card
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

  // Press ESC key to close action card
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

  // Clear all timers
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
        // Prevent browser context menu from appearing on panel
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

      {/* Action card rendering */}
      {selectedMessageId && (
        <MessageActionCard
          messageId={selectedMessageId}
          position={actionCardPosition}
          onAction={(action) => handleAction(action, selectedMessageId)}
          onClose={closeActionCard}
        />
      )}

      {/* Bottom spacing */}
      <div className="h-12"></div>
    </div>
  );
};

export default MessagePanel;
