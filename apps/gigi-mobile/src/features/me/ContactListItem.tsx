import React, { useCallback, useEffect, useState } from 'react'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { getAvatarUrl } from '@/utils/imageStorage'
import { formatShortPeerId } from '@/utils/peerUtils'

interface ContactListItemProps {
  name: string
  peerId: string
  onClick: () => void
}

const ContactListItem: React.FC<ContactListItemProps> = ({
  name,
  peerId,
  onClick,
}) => {
  const [avatarUrl, setAvatarUrl] = useState<string | null>(null)

  const loadAvatar = useCallback(async () => {
    if (peerId) {
      const url = await getAvatarUrl(peerId)
      setAvatarUrl(url)
    }
  }, [peerId])

  useEffect(() => {
    loadAvatar()
  }, [loadAvatar])

  return (
    <div
      className="flex items-center p-4 hover:bg-gray-50 cursor-pointer transition-colors duration-200 border-b border-gray-100 last:border-b-0"
      onClick={onClick}
    >
      <div className="w-12 h-12 rounded-full overflow-hidden mr-4 flex-shrink-0">
        <Avatar className="w-full h-full">
          <AvatarImage
            src={avatarUrl || ''}
            alt={name}
            className="object-cover"
          />
          <AvatarFallback className="bg-gradient-to-br from-green-400 to-green-600 text-white font-bold">
            {name?.charAt(0).toUpperCase() || '?'}
          </AvatarFallback>
        </Avatar>
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex justify-between items-start">
          <h3 className="font-semibold text-gray-900 truncate text-lg">
            {name}
          </h3>
          <svg
            className="w-4 h-4 text-gray-400 ml-2 flex-shrink-0"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M9 5l7 7-7 7"
            ></path>
          </svg>
        </div>
        <p className="text-gray-500 text-sm font-mono truncate mt-1">
          {formatShortPeerId(peerId)}
        </p>
      </div>
    </div>
  )
}

export default ContactListItem
