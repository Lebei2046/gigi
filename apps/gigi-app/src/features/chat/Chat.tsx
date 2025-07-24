import { useParams } from 'react-router-dom';
import ChatPanel from './components/ChatPanel';
import { initialMessages } from '../../data/messages';

export default function Chat() {
  const { id } = useParams<{ id: string }>();

  return (
    <div className="h-screen w-full bg-gray-50 relative">
      <ChatPanel groupName={`聊天 ${id || ''}`} initialMessages={initialMessages} />
    </div>
  );
}
