import type { Message } from '@/store/chatRoomSlice'
import TextMessageBubble from './TextMessageBubble'
import ImageMessageBubble from './ImageMessageBubble'
import FileMessageBubble from '@/components/FileMessageBubble'

interface MessageBubbleProps {
  message: Message
  onDownloadRequest?: (
    messageId: string,
    shareCode: string,
    filename: string
  ) => void
}

export default function MessageBubble({
  message,
  onDownloadRequest,
}: MessageBubbleProps) {
  console.log('💬 MessageBubble render:', {
    id: message.id,
    messageType: message.messageType,
    hasImageData: !!message.imageData,
    content: message.content?.substring(0, 50),
  })

  return (
    <>
      {/* Show sender name for incoming messages */}
      {!message.isOutgoing && (
        <p className="text-xs font-medium mb-1 opacity-70">
          {message.isGroup && '👥'} {message.from_nickname}
        </p>
      )}

      {/* Render message content based on type */}
      {message.messageType === 'image' ? (
        <ImageMessageBubble message={message} />
      ) : message.messageType === 'file' ? (
        <FileMessageBubble
          message={message}
          onDownloadRequest={onDownloadRequest}
        />
      ) : (
        <TextMessageBubble message={message} />
      )}
    </>
  )
}
