import { useParams } from 'react-router-dom';
import ChatPanel from './components/ChatPanel';
import { useMessagesByChatId } from '../../models/message';
import { useChat } from '../../models/chat';

export default function Chat() {
  const { id } = useParams<{ id: string }>();
  const chatId = id ? parseInt(id) : 0;
  const messages = useMessagesByChatId(chatId);
  const chat = useChat(chatId);

  return (
    <div className="h-screen w-full bg-gray-50 relative">
      <ChatPanel chatId={chatId} groupName={`${chat?.name}`} initialMessages={messages || []} />
    </div>
  );
}
