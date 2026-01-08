import type { Group } from '@/models/db'
import { Trash2 } from 'lucide-react'

interface GroupCardProps {
  group: Group
  latestMessage?: string
  unreadCount: number
  onGroupClick: (groupId: string) => void
  onShare: (group: Group) => void
  onClearMessages: (groupId: string, groupName: string) => void
}

export function GroupCard({
  group,
  latestMessage,
  unreadCount,
  onGroupClick,
  onShare,
  onClearMessages,
}: GroupCardProps) {
  return (
    <div className="bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-md transition-shadow">
      <div
        className="flex justify-between items-start p-4 cursor-pointer"
        onClick={() => onGroupClick(group.id)}
      >
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <div className="w-10 h-10 bg-blue-100 rounded-full flex items-center justify-center">
              <span className="text-blue-600 font-semibold">G</span>
            </div>
            <div className="flex-1">
              <div className="flex items-center gap-2">
                <span className="font-semibold text-gray-900">
                  {group.name}
                </span>
                {unreadCount > 0 && (
                  <span className="bg-red-500 text-white text-xs font-bold px-2 py-0.5 rounded-full min-w-[20px] text-center">
                    {unreadCount}
                  </span>
                )}
              </div>
              <div className="flex items-center gap-2 text-xs text-gray-500">
                <span className="bg-gray-100 px-2 py-0.5 rounded">
                  {group.joined ? 'Member' : 'Owner'}
                </span>
              </div>
            </div>
          </div>
          {latestMessage && (
            <div className="text-sm text-gray-600 mt-2 truncate ml-12">
              💬 {latestMessage}
            </div>
          )}
        </div>
        <button
          onClick={e => {
            e.stopPropagation()
            onShare(group)
          }}
          className="p-2 text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
        >
          <svg
            className="w-5 h-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m9.032 4.026a3 3 0 10-4.732 2.684m4.732-2.684a3 3 0 00-4.732-2.684"
            ></path>
          </svg>
        </button>
        <button
          onClick={e => {
            e.stopPropagation()
            onClearMessages(group.id, group.name)
          }}
          className="p-2 text-red-600 hover:bg-red-50 rounded-lg transition-colors"
          title="Clear messages"
        >
          <Trash2 className="w-5 h-5" />
        </button>
      </div>
    </div>
  )
}
