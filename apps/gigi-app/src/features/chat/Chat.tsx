import ChatPanel from './components/ChatPanel';
import { initialMessages } from './data/messages';

export default function Chat() {
  return (
    <div className="h-screen max-w-md mx-auto bg-gray-50 relative">
      <ChatPanel groupName="乒羽网约球" initialMessages={initialMessages} />
    </div>
  );
}