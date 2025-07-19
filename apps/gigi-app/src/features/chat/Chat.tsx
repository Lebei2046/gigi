import { useParams } from 'react-router-dom';
import ChatPanel from './components/ChatPanel';
import { initialMessages } from '../../data/messages';

export default function Chat() {
  const { chatId } = useParams<{ chatId: string }>();

  return (
    <div className="h-screen max-w-md mx-auto bg-gray-50 relative">
      <ChatPanel groupName={`聊天 ${chatId || ''}`} initialMessages={initialMessages} />
    </div>
  );
}