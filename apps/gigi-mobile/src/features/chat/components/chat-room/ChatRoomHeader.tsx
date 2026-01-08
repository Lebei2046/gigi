import { Button } from '@/components/ui/button'
import { ArrowLeft as BackIcon } from 'lucide-react'
import { formatShortPeerId } from '@/utils/peerUtils'

interface ChatRoomHeaderProps {
  chatTitle: string
  chatId?: string
  isGroupChat: boolean
  onGoBack: () => void
}

export default function ChatRoomHeader({
  chatTitle,
  chatId,
  isGroupChat,
  onGoBack,
}: ChatRoomHeaderProps) {
  return (
    <div className="flex items-center p-4 border-b bg-white">
      <Button variant="ghost" size="sm" onClick={onGoBack} className="mr-3">
        <BackIcon size={20} />
      </Button>
      <div>
        <h2 className="text-lg font-semibold">
          {isGroupChat ? `👥 ${chatTitle}` : chatTitle}
        </h2>
        <p className="text-sm text-gray-500">
          {isGroupChat
            ? `Group • ${chatId ? formatShortPeerId(chatId) : 'N/A'}`
            : `Direct • ${chatId ? formatShortPeerId(chatId) : 'N/A'}`}
        </p>
      </div>
    </div>
  )
}
