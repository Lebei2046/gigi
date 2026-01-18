import type { Peer } from '@/utils/messaging'
import { Trash2 } from 'lucide-react'

interface PeerCardProps {
  peer: Peer
  latestMessage?: string
  unreadCount: number
  lastMessageTime?: string
  onPeerClick: (peer: Peer) => void
  onClearMessages: (peerId: string, peerNickname: string) => void
}

export function PeerCard({
  peer,
  latestMessage,
  unreadCount,
  lastMessageTime,
  onPeerClick,
  onClearMessages,
}: PeerCardProps) {
  return (
    <div className="bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-md transition-all cursor-pointer hover:border-green-300">
      <div className="p-4">
        <div className="flex justify-between items-start">
          <div
            onClick={() => onPeerClick(peer)}
            className="flex items-start gap-3 flex-1 cursor-pointer"
          >
            <div className="w-12 h-12 bg-gradient-to-br from-green-400 to-green-600 rounded-full flex items-center justify-center flex-shrink-0">
              <span className="text-white font-bold text-lg">
                {peer.nickname?.charAt(0).toUpperCase() || '?'}
              </span>
            </div>
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2 mb-1">
                <span className="font-semibold text-gray-900">
                  {peer.nickname}
                </span>
                {unreadCount > 0 && (
                  <span className="bg-red-500 text-white text-xs font-bold px-2 py-0.5 rounded-full min-w-[20px] text-center">
                    {unreadCount}
                  </span>
                )}
              </div>
              <div className="text-xs text-gray-500 font-mono truncate">
                {peer.id}
              </div>
              {latestMessage && (
                <div className="text-sm text-gray-600 truncate mt-1">
                  {latestMessage}
                </div>
              )}
              {peer.capabilities.length > 0 && (
                <div className="flex flex-wrap gap-1 mt-1">
                  {peer.capabilities.map((cap, index) => (
                    <span
                      key={index}
                      className="bg-gray-100 text-gray-600 text-xs px-2 py-0.5 rounded"
                    >
                      {cap}
                    </span>
                  ))}
                </div>
              )}
            </div>
          </div>
          <div className="text-right ml-3 flex-shrink-0 flex flex-col items-end">
            {lastMessageTime && (
              <div className="text-xs text-gray-400 mb-1">
                {lastMessageTime}
              </div>
            )}
            <button
              onClick={e => {
                e.stopPropagation()
                onClearMessages(peer.id, peer.nickname || 'Unknown')
              }}
              className="p-2 text-red-600 hover:bg-red-50 rounded-lg transition-colors"
              title="Clear messages"
            >
              <Trash2 className="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
