import React from "react";
import { HiOutlineSearch, HiPlusCircle } from "react-icons/hi";
import ChatListItem from "../components/ChatListItem";
import { allChats } from "../../../data/users";

interface ChatListProps {
  onChatSelect: (id: string) => void;
}

const ChatList: React.FC<ChatListProps> = ({ onChatSelect }) => {
  return (
    <div className="flex flex-col h-full">
      {/* 顶部导航栏 - 已修复标题居中问题 */}
      <div className="sticky top-0 z-10 bg-gray-100 px-4 py-3 flex items-center">
        {/* 左侧占位元素 */}
        <div className="flex-1"></div>

        {/* 居中标题 */}
        <div className="flex-1 flex justify-center">
          <span className="text-xl font-semibold">唧唧</span>
        </div>

        {/* 右侧图标 */}
        <div className="flex-1 flex justify-end space-x-3">
          <HiOutlineSearch className="w-6 h-6 text-gray-600" />
          <HiPlusCircle className="w-6 h-6 text-gray-600" />
        </div>
      </div>

      {/* 聊天列表 */}
      <div className="flex-1 overflow-y-auto">
        {allChats.map((chat) => (
          <ChatListItem
            key={chat.id}
            id={chat.id}
            name={chat.name}
            lastMessage={chat.lastMessage || "暂无消息"}
            time={chat.lastMessageTime || ""}
            unreadCount={chat.unreadCount}
            isGroup={chat.isGroup || false}
            onClick={() => onChatSelect(chat.id)}
          />
        ))}
      </div>
    </div>
  );
};

export default ChatList;
