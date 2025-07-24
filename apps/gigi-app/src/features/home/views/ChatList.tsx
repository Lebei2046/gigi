import React, { useState } from "react";
import ChatListItem from "../components/ChatListItem";
import { allChats } from "../../../data/users";
import TopBar from "../components/TopBar";

interface ChatListProps {
  onChatSelect: (id: string) => void;
}

const ChatList: React.FC<ChatListProps> = ({ onChatSelect }) => {
  const [menuOpen, setMenuOpen] = useState(false);
  const [isMenuClosing, setIsMenuClosing] = useState(false);
  const closingTimeoutRef = React.useRef<NodeJS.Timeout | null>(null);

  const setMenuOpenWithDelay = (open: boolean) => {
    // 清除之前的定时器
    if (closingTimeoutRef.current) {
      clearTimeout(closingTimeoutRef.current);
      closingTimeoutRef.current = null;
    }

    if (!open) {
      // 设置即将关闭状态
      setIsMenuClosing(true);

      // 延迟更新菜单状态，给点击事件处理留出时间
      closingTimeoutRef.current = setTimeout(() => {
        setMenuOpen(false);
        setIsMenuClosing(false);
      }, 100);
    } else {
      // 直接打开菜单
      setMenuOpen(true);
      setIsMenuClosing(false);
    }
  };

  // 清理定时器
  React.useEffect(() => {
    return () => {
      if (closingTimeoutRef.current) {
        clearTimeout(closingTimeoutRef.current);
      }
    };
  }, []);

  return (
    <div className="flex flex-col h-full">
      <TopBar title="唧唧" menuOpen={menuOpen} setMenuOpen={setMenuOpenWithDelay} />

      {/* 聊天列表 */}
      <div className="flex-1 overflow-y-auto">
        {allChats.map((chat) => (
          <ChatListItem
            key={chat.id}
            id={chat.id}
            name={chat.name}
            lastMessage={chat.lastMessage || "暂无消息"}
            time={chat.lastMessageTime || ""}
            unreadCount={/*chat.unreadCount || */ 0}
            isGroup={/*chat.isGroup || */ false}
            onClick={() => onChatSelect(chat.id)}
            menuOpen={menuOpen || isMenuClosing}
          />
        ))}
      </div>
    </div>
  );
};

export default ChatList;
