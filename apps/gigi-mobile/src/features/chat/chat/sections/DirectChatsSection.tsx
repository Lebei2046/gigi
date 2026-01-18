import type { Peer } from '@/utils/messaging'
import type { Conversation } from '@/utils/conversationUtils'
import { PeerCard } from '../cards'
import DirectChatsEmptyState from '../cards/DirectChatsEmptyState'
import { ensureMilliseconds, isValidTimestamp } from '@/utils/conversationUtils'

interface DirectChatsSectionProps {
  peers: Peer[]
  conversations: Conversation[]
  latestMessages: Record<string, string>
  onPeerClick: (peer: Peer) => void
  onClearMessages: (
    chatId: string,
    isGroupChat: boolean,
    chatName: string
  ) => void
}

export default function DirectChatsSection({
  peers,
  conversations,
  latestMessages,
  onPeerClick,
  onClearMessages,
}: DirectChatsSectionProps) {
  return (
    <div>
      <div className="flex items-center gap-2 mb-3">
        <span className="text-lg">💬</span>
        <h3 className="text-lg font-semibold text-gray-900">Direct Chats</h3>
        <span className="bg-green-100 text-green-600 text-xs font-medium px-2 py-1 rounded-full">
          {peers.length}
        </span>
      </div>
      {peers.length === 0 ? (
        <DirectChatsEmptyState />
      ) : (
        <div className="space-y-3">
          {peers.map(peer => {
            const conversationInfo = conversations.find(c => c.id === peer.id)
            const unreadCount = conversationInfo?.unread_count || 0
            const timestamp = conversationInfo?.last_message_timestamp

            const lastMessageTime = timestamp && isValidTimestamp(timestamp)
              ? new Date(typeof timestamp === 'string' ? timestamp : ensureMilliseconds(timestamp)).toLocaleString()
              : undefined

            return (
              <PeerCard
                key={peer.id}
                peer={peer}
                latestMessage={latestMessages[peer.id]}
                unreadCount={unreadCount}
                lastMessageTime={lastMessageTime}
                onPeerClick={onPeerClick}
                onClearMessages={(peerId, peerNickname) =>
                  onClearMessages(peerId, false, peerNickname)
                }
              />
            )
          })}
        </div>
      )}
    </div>
  )
}
