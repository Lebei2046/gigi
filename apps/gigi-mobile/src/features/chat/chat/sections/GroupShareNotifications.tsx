import type { GroupShareMessage } from '@/utils/messaging'

interface GroupShareNotificationsProps {
  notifications: GroupShareMessage[]
  onAccept: (notification: GroupShareMessage) => void
  onIgnore: (notification: GroupShareMessage) => void
}

export default function GroupShareNotifications({
  notifications,
  onAccept,
  onIgnore,
}: GroupShareNotificationsProps) {
  if (notifications.length === 0) {
    return null
  }

  return (
    <div className="mb-6 space-y-3">
      {notifications.map(notification => (
        <div
          key={notification.from_peer_id}
          className="bg-gradient-to-r from-purple-50 to-pink-50 border border-purple-200 rounded-xl p-4 shadow-sm"
        >
          <div className="flex justify-between items-start">
            <div className="flex-1">
              <div className="flex items-center gap-2 mb-1">
                <span className="text-lg">🎉</span>
                <span className="font-semibold text-purple-800">
                  Group Invitation
                </span>
              </div>
              <div className="text-sm text-gray-700 mb-1">
                from{' '}
                <span className="font-medium">
                  {notification.from_nickname}
                </span>
              </div>
              <div className="text-sm font-medium text-purple-600 bg-purple-100 inline-block px-2 py-1 rounded">
                {notification.group_name}
              </div>
            </div>
          </div>
          <div className="flex gap-2 mt-3">
            <button
              onClick={() => onAccept(notification)}
              className="flex-1 py-2 bg-green-600 hover:bg-green-700 text-white text-sm font-medium rounded-lg transition-colors"
            >
              Accept
            </button>
            <button
              onClick={() => onIgnore(notification)}
              className="flex-1 py-2 bg-gray-200 hover:bg-gray-300 text-gray-700 text-sm font-medium rounded-lg transition-colors"
            >
              Ignore
            </button>
          </div>
        </div>
      ))}
    </div>
  )
}
