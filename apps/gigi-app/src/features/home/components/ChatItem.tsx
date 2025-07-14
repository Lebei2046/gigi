import React from "react";
import Avatar from "./Avatar";

interface ChatItemProps {
  id: string;
  name: string;
  lastMessage: string;
  time: string;
  unreadCount?: number;
  isGroup: boolean;
}

const ChatItem: React.FC<ChatItemProps> = ({
  name,
  lastMessage,
  time,
  unreadCount,
  isGroup,
}) => {
  return (
    <div className="flex items-center py-3 px-4 hover:bg-gray-50 active:bg-gray-100">
      <div className="flex-shrink-0 mr-3">
        <Avatar name={name} isGroup={isGroup} />
      </div>

      <div className="flex-1 min-w-0">
        <div className="flex justify-between items-center">
          <h3 className="font-medium text-gray-900 truncate">{name}</h3>
          <span className="text-xs text-gray-500">{time}</span>
        </div>

        <div className="flex justify-between mt-1">
          <p className="text-sm text-gray-500 truncate max-w-[70%]">
            {lastMessage}
          </p>
          {unreadCount && unreadCount > 0 && (
            <span className="badge badge-primary badge-sm min-w-[22px] h-5">
              {unreadCount > 99 ? "99+" : unreadCount}
            </span>
          )}
        </div>
      </div>
    </div>
  );
};

export default ChatItem;
