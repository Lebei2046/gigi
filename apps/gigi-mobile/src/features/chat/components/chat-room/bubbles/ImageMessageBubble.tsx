import type { Message } from '@/store/chatRoomSlice'

interface ImageMessageBubbleProps {
  message: Message
}

export default function ImageMessageBubble({
  message,
}: ImageMessageBubbleProps) {
  console.log('🖼️ ImageMessageBubble render:', {
    messageType: message.messageType,
    imageData: message.imageData
      ? `data length: ${message.imageData.length}`
      : 'no data',
    content: message.content,
    filename: message.filename,
  })

  return (
    <div className="flex flex-col gap-2">
      {message.imageData && (
        <img
          src={message.imageData}
          alt={message.filename}
          className="max-w-xs max-h-48 rounded-lg object-cover"
          onError={e => console.error('❌ Image load error:', e)}
          onLoad={() => console.log('✅ Image loaded successfully')}
        />
      )}
      <p className="text-sm break-words">{message.content}</p>
    </div>
  )
}
