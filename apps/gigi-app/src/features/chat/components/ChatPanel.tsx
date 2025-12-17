import { useState, useEffect, useRef } from 'react';
import TopBar from './TopBar';
import MessagePanel from './MessagePanel';
import InputBar from './InputBar';
import type { Message } from '../../../models/db';
import { addMessage as addMessageToDb } from '../../../models/message';

interface ChatPanelProps {
  chatId: string;
  groupName: string;
  initialMessages: Message[];
}

const ChatPanel = ({ chatId, groupName, initialMessages }: ChatPanelProps) => {
  const [messages, setMessages] = useState<Message[]>([]);
  const panelRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [inputHeight, setInputHeight] = useState(64); // Initial input area height estimate

  // Card height state
  const [cardHeight, setCardHeight] = useState(0);

  // Update messages state when initialMessages changes
  useEffect(() => {
    setMessages(initialMessages);
  }, [initialMessages]);

  // Provide message management methods to child components
  const handleMessageAction = (action: string, messageId: number) => {
    if (action === 'Delete') {
      setMessages((prev) => prev.filter((msg) => msg.id !== messageId));
    }
  };

  // Scroll to bottom
  const scrollToBottom = () => {
    if (panelRef.current) {
      panelRef.current.scrollTop = panelRef.current.scrollHeight;
    }
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  // Add text message
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
      // Display message in UI even if database storage fails
      setMessages((prev) => [...prev, { ...newMessage, id: Date.now() }]);
    }

    setCardHeight(0); // Close card after sending message

    // Delayed scroll to bottom
    setTimeout(() => {
      scrollToBottom();
    }, 100);
  };

  // Add image message
  const addImageMessage = async (imageId: string) => {
    const newMessage: Omit<Message, 'id'> = {
      chatId,
      sender: 'lebei',
      content: `[image:${imageId}]`, // Use special format to identify image messages
      timestamp: new Date(),
    };

    try {
      const id = await addMessageToDb(newMessage);
      setMessages((prev) => [...prev, { ...newMessage, id }]);
    } catch (error) {
      console.error('Failed to send image message:', error);
      // Display message in UI even if database storage fails
      setMessages((prev) => [...prev, { ...newMessage, id: Date.now() }]);
    }

    setCardHeight(0); // Close card after sending message

    // Delayed scroll to bottom
    setTimeout(() => {
      scrollToBottom();
    }, 100);
  };

  // Handle card state change
  const handleCardHeightChange = (height: number) => {
    setCardHeight(height);

    // Delayed scroll to bottom
    setTimeout(() => {
      scrollToBottom();
    }, 350);
  };

  // Handle input area height change
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
      {/* Top bar */}
      <div className="sticky top-0 z-10 bg-white shadow-sm w-full">
        <TopBar groupName={groupName} onBack={() => window.history.back()} />
      </div>

      {/* Message panel */}
      <div ref={panelRef} className="flex-1 overflow-auto bg-white p-4 w-full">
        <MessagePanel
          messages={messages}
          me="lebei"
          onMessageAction={handleMessageAction}
        />
      </div>

      {/* Bottom input area - using absolute positioning */}
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
