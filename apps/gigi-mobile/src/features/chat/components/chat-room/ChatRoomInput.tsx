import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'

interface ChatRoomInputProps {
  newMessage: string
  sending: boolean
  isGroupChat: boolean
  chatName?: string
  onSendMessage: () => void
  onFileSelect: () => void
  onImageSelect: () => void
  onMessageChange: (e: React.ChangeEvent<HTMLInputElement>) => void
  onKeyDown: (e: React.KeyboardEvent) => void
}

export default function ChatRoomInput({
  newMessage,
  sending,
  isGroupChat,
  chatName,
  onSendMessage,
  onFileSelect,
  onImageSelect,
  onMessageChange,
  onKeyDown,
}: ChatRoomInputProps) {
  return (
    <div className="border-t bg-white p-4">
      <div className="flex gap-2">
        {/* File upload button */}
        <Button
          variant="outline"
          size="sm"
          onClick={onFileSelect}
          disabled={sending}
          title="Select any file"
        >
          📎
        </Button>

        {/* Image upload button */}
        <Button
          variant="outline"
          size="sm"
          onClick={onImageSelect}
          disabled={sending}
          title="Select image file"
        >
          📷
        </Button>

        <Input
          value={newMessage}
          onChange={onMessageChange}
          onKeyDown={onKeyDown}
          placeholder={chatName ? `Message ${chatName}...` : 'Message...'}
          disabled={sending}
          className="flex-1"
        />
        <Button
          onClick={onSendMessage}
          disabled={!newMessage.trim() || sending}
          size="sm"
        >
          {sending ? 'Sending...' : 'Send'}
        </Button>
      </div>
    </div>
  )
}
