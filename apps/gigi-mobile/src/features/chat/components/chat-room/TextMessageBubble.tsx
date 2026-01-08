import type { Message } from '@/store/chatRoomSlice'

interface TextMessageBubbleProps {
  message: Message
}

export default function TextMessageBubble({ message }: TextMessageBubbleProps) {
  return <p className="text-sm break-words">{message.content}</p>
}
