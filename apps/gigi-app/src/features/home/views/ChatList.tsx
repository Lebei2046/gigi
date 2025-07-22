import React from "react";
import ChatListItem from "../components/ChatListItem";
import { allChats } from "../../../data/users";
import TopBar from "../components/TopBar";

interface ChatListProps {
  onChatSelect: (id: string) => void;
}

const ChatList: React.FC<ChatListProps> = ({ onChatSelect }) => {
  return (
    <div className="flex flex-col h-full">
      <TopBar title="唧唧" />

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
          />
        ))}
      </div>
    </div>
  );
};

export default ChatList;
