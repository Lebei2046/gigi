import type { Group } from '@/models/db'
import type { Conversation } from '@/utils/conversationUtils'
import { GroupCard } from '../cards'

interface GroupsSectionProps {
  groups: Group[]
  conversations: Conversation[]
  latestMessages: Record<string, string>
  onGroupClick: (groupId: string) => void
  onShare: (group: Group) => void
  onClearMessages: (
    chatId: string,
    isGroupChat: boolean,
    chatName: string
  ) => void
}

export default function GroupsSection({
  groups,
  conversations,
  latestMessages,
  onGroupClick,
  onShare,
  onClearMessages,
}: GroupsSectionProps) {
  if (groups.length === 0) {
    return null
  }

  const totalGroupUnread = groups.reduce((sum, group) => {
    const conversationInfo = conversations.find(c => c.id === group.id)
    return sum + (conversationInfo?.unread_count || 0)
  }, 0)

  return (
    <div className="mb-6">
      <div className="flex items-center gap-2 mb-3">
        <span className="text-lg">👥</span>
        <h3 className="text-lg font-semibold text-gray-900">Groups</h3>
        <span className="bg-blue-100 text-blue-600 text-xs font-medium px-2 py-1 rounded-full">
          {groups.length}
        </span>
        {totalGroupUnread > 0 && (
          <span className="bg-red-500 text-white text-xs font-bold px-2 py-1 rounded-full">
            {totalGroupUnread}
          </span>
        )}
      </div>
      <div className="space-y-3">
        {groups.map(group => {
          const conversationInfo = conversations.find(c => c.id === group.id)
          const unreadCount = conversationInfo?.unread_count || 0

          return (
            <GroupCard
              key={group.id}
              group={group}
              latestMessage={latestMessages[group.id]}
              unreadCount={unreadCount}
              onGroupClick={onGroupClick}
              onShare={onShare}
              onClearMessages={(groupId, groupName) =>
                onClearMessages(groupId, true, groupName)
              }
            />
          )
        })}
      </div>
    </div>
  )
}
