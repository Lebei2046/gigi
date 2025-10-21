import type { Sender } from '../../../../data/senders';
import type { Message } from '../../../../models/db';
import Avatar from '../../../../components/Avatar';
const MessageBubble = ({
  message,
  sender,
  isMe,
}: {
  message: Message;
  sender: Sender;
  isMe: boolean;
}) => (
  <div className={`flex mb-4 ${isMe ? 'justify-end' : ''}`}>
    {!isMe && (
      <span className="w-10 h-10 rounded-full mr-2 flex items-center justify-center bg-gray-200">
        <Avatar
          name={sender.name}
          size={40}
          address={sender.id}
        />
      </span>
    )}

    <div className={isMe ? 'flex-col items-end' : 'flex-col'}>
      {!isMe && <div className="text-sm mb-1">{sender.name}</div>}

      <div
        className={`
        max-w-xs md:max-w-md px-4 py-2 rounded-2xl
        ${isMe
            ? 'bg-blue-500 text-white rounded-tr-none'
            : 'bg-gray-200 rounded-tl-none'
          }
      `}
      >
        {message.content}
      </div>

      <div
        className={`text-xs opacity-50 mt-1 ${isMe ? 'text-right' : ''}`}
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
