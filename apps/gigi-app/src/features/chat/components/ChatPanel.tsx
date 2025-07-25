import { useState, useEffect, useRef } from 'react';
import TopBar from './TopBar';
import MessagePanel from './MessagePanel';
import InputBar from './InputBar';
import type { Message } from '../../../models/db';
import { addMessage as addMessageToDb } from '../../../models/message';

interface ChatPanelProps {
  chatId: number;
  groupName: string;
  initialMessages: Message[];
}

const ChatPanel = ({ chatId, groupName, initialMessages }: ChatPanelProps) => {
  const [messages, setMessages] = useState<Message[]>([]);
  const panelRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [inputHeight, setInputHeight] = useState(64); // 初始输入区高度估算值

  // 卡片高度状态
  const [cardHeight, setCardHeight] = useState(0);

  // 当 initialMessages 变化时，更新 messages 状态
  useEffect(() => {
    setMessages(initialMessages);
  }, [initialMessages]);

  // 提供消息管理方法给子组件
  const handleMessageAction = (action: string, messageId: number) => {
    if (action === '删除') {
      setMessages((prev) => prev.filter((msg) => msg.id !== messageId));
    }
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

  // 添加文本消息
  const addTextMessage = async (content: string) => {
    const newMessage: Omit<Message, 'id'> = {
      chatId,
      sender: 'lebei',
      content,
      timestamp: new Date(),
    };

    try {
      const id = await addMessageToDb(newMessage);
      setMessages((prev) => [...prev, { ...newMessage, id }]);
    } catch (error) {
      console.error('Failed to send message:', error);
      // 即使数据库存储失败，也在UI上显示消息
      setMessages((prev) => [...prev, { ...newMessage, id: Date.now() }]);
    }

    setCardHeight(0); // 发送消息后关闭卡片

    // 延迟滚动到底部
    setTimeout(() => {
      scrollToBottom();
    }, 100);
  };

  // 添加图片消息
  const addImageMessage = async (imageId: string) => {
    const newMessage: Omit<Message, 'id'> = {
      chatId,
      sender: 'lebei',
      content: `[image:${imageId}]`, // 使用特殊格式标识图片消息
      timestamp: new Date(),
    };

    try {
      const id = await addMessageToDb(newMessage);
      setMessages((prev) => [...prev, { ...newMessage, id }]);
    } catch (error) {
      console.error('Failed to send image message:', error);
      // 即使数据库存储失败，也在UI上显示消息
      setMessages((prev) => [...prev, { ...newMessage, id: Date.now() }]);
    }

    setCardHeight(0); // 发送消息后关闭卡片

    // 延迟滚动到底部
    setTimeout(() => {
      scrollToBottom();
    }, 100);
  };

  // 处理卡片状态变化
  const handleCardHeightChange = (height: number) => {
    setCardHeight(height);

    // 延迟滚动到底部
    setTimeout(() => {
      scrollToBottom();
    }, 350);
  };

  // 处理输入区高度变化
  const handleInputHeightChange = (height: number) => {
    setInputHeight(height);
    setTimeout(() => scrollToBottom(), 50);
  };

  return (
    <div
      ref={containerRef}
      className="flex flex-col h-screen overflow-hidden w-full"
      style={{
        paddingBottom: `${inputHeight + cardHeight}px`,
        transition: 'padding-bottom 300ms ease',
      }}
    >
      {/* 顶部栏 */}
      <div className="sticky top-0 z-10 bg-white shadow-sm w-full">
        <TopBar groupName={groupName} onBack={() => window.history.back()} />
      </div>

      {/* 消息面板 */}
      <div ref={panelRef} className="flex-1 overflow-auto bg-white p-4 w-full">
        <MessagePanel
          messages={messages}
          currentUserId="lebei"
          onMessageAction={handleMessageAction}
        />
      </div>

      {/* 底部输入区 - 使用绝对定位 */}
      <div
        className="fixed bottom-0 left-0 right-0 z-20 transition-all duration-300 w-full"
        style={{
          height: `${inputHeight}px`,
          transform: `translateY(${cardHeight > 0 ? -cardHeight : 0}px)`,
        }}
      >
        <InputBar
          onSend={addTextMessage}
          onImageSend={addImageMessage}
          onCardHeightChange={handleCardHeightChange}
          onInputHeightChange={handleInputHeightChange}
        />
      </div>
    </div>
  );
};

export default ChatPanel;
