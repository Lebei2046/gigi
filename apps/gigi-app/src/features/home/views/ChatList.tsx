import React, { useState, useEffect } from "react";
import ChatListItem from "../components/ChatListItem";
import TopBar from "../components/TopBar";
import { useAllChats } from "../../../models/chat";

interface ChatListProps {
  onChatSelect: (id: string) => void;
}

const ChatList: React.FC<ChatListProps> = ({ onChatSelect }) => {
  const [menuOpen, setMenuOpen] = useState(false);
  const [isMenuClosing, setIsMenuClosing] = useState(false);
  const closingTimeoutRef = React.useRef<NodeJS.Timeout | null>(null);
  const chats = useAllChats();

  const setMenuOpenWithDelay = (open: boolean) => {
    // Clear previous timer
    if (closingTimeoutRef.current) {
      clearTimeout(closingTimeoutRef.current);
      closingTimeoutRef.current = null;
    }

    if (!open) {
      // Set about to close state
      setIsMenuClosing(true);

      // Delay menu state update to allow time for click event handling
      closingTimeoutRef.current = setTimeout(() => {
        setMenuOpen(false);
        setIsMenuClosing(false);
      }, 100);
    } else {
      // Open menu directly
      setMenuOpen(true);
      setIsMenuClosing(false);
    }
  };

  // Clean up timer
  useEffect(() => {
    return () => {
      if (closingTimeoutRef.current) {
        clearTimeout(closingTimeoutRef.current);
      }
    };
  }, []);

  return (
    <div className="flex flex-col h-full">
      <TopBar title="Giji" menuOpen={menuOpen} setMenuOpen={setMenuOpenWithDelay} />

      {/* Chat list */}
      <div className="flex-1 overflow-y-auto">
        {chats && chats.length > 0 ?
          (
            chats.map((chat) => (
              <ChatListItem
                key={chat.id}
                id={chat.id}
                name={chat.name}
                lastMessage={chat.lastMessage || "No messages yet"}
                time={chat.lastMessageTime || ""}
                unreadCount={chat.unreadCount || 0}
                onClick={() => onChatSelect(chat.id)}
                menuOpen={menuOpen || isMenuClosing}
              />
            ))
          )
          : (<div className="text-center text-gray-500 py-4">No data available</div>)
        }
      </div>
    </div>
  );
};

export default ChatList;
