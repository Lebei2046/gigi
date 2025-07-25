import type { User } from '../../types';
import type { Message } from '../../../../models/db';

const MessageBubble = ({
  message,
  sender,
  isCurrentUser,
}: {
  message: Message;
  sender: User;
  isCurrentUser: boolean;
}) => (
  <div className={`flex mb-4 ${isCurrentUser ? 'justify-end' : ''}`}>
    {!isCurrentUser && (
      <span className="w-10 h-10 rounded-full mr-2 flex items-center justify-center bg-gray-200">
        {sender.avatar && <sender.avatar size={40} />}
      </span>
    )}

    <div className={isCurrentUser ? 'flex-col items-end' : 'flex-col'}>
      {!isCurrentUser && <div className="text-sm mb-1">{sender.name}</div>}

      <div
        className={`
        max-w-xs md:max-w-md px-4 py-2 rounded-2xl
        ${isCurrentUser
            ? 'bg-blue-500 text-white rounded-tr-none'
            : 'bg-gray-200 rounded-tl-none'
          }
      `}
      >
        {message.content}
      </div>

      <div
        className={`text-xs opacity-50 mt-1 ${isCurrentUser ? 'text-right' : ''}`}
      >
        {new Date(message.timestamp).toLocaleTimeString([], {
          hour: '2-digit',
          minute: '2-digit',
        })}
      </div>
    </div>
  </div>
);

export default MessageBubble;
