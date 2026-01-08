import type { Peer } from '@/utils/messaging'
import type { Chat } from '@/models/db'
import { PeerCard } from '../cards'
import DirectChatsEmptyState from '../cards/DirectChatsEmptyState'

interface DirectChatsSectionProps {
  peers: Peer[]
  chats: Chat[]
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
  chats,
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
            const chatInfo = chats.find(chat => chat.id === peer.id)
            const unreadCount = chatInfo?.unreadCount || 0

            return (
              <PeerCard
                key={peer.id}
                peer={peer}
                latestMessage={latestMessages[peer.id]}
                unreadCount={unreadCount}
                lastMessageTime={chatInfo?.lastMessageTime}
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
