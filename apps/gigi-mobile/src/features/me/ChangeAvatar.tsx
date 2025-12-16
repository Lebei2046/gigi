import { useCallback, useEffect, useRef, useState } from 'react'
import { FiCamera } from 'react-icons/fi'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { getAvatarUrl, storeAvatar } from '@/utils/imageStorage'

interface ChangeAvatarProps {
  peerId: string
  name?: string
}

export default function ChangeAvatar({ peerId, name }: ChangeAvatarProps) {
  const [avatarUrl, setAvatarUrl] = useState<string | null>(null)
  const [isUploading, setIsUploading] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)

  const loadAvatar = useCallback(async () => {
    if (peerId) {
      const url = await getAvatarUrl(peerId)
      setAvatarUrl(url)
    }
  }, [peerId])

  useEffect(() => {
    loadAvatar()
  }, [loadAvatar])

  const handleAvatarClick = () => {
    fileInputRef.current?.click()
  }

  const handleAvatarUpload = async (
    event: React.ChangeEvent<HTMLInputElement>
  ) => {
    const file = event.target.files?.[0]
    if (file && file.type.startsWith('image/') && peerId) {
      try {
        setIsUploading(true)
        await storeAvatar(peerId, file)
        await loadAvatar()
      } catch (error) {
        console.error('Failed to upload avatar:', error)
        // Show a more subtle error notification
        const notification = document.createElement('div')
        notification.textContent = 'Failed to upload avatar'
        notification.className =
          'fixed top-4 right-4 bg-red-500 text-white px-4 py-2 rounded-lg shadow-lg z-50 transition-opacity duration-300'
        document.body.appendChild(notification)

        setTimeout(() => {
          notification.style.opacity = '0'
          setTimeout(() => document.body.removeChild(notification), 300)
        }, 3000)
      } finally {
        setIsUploading(false)
        if (fileInputRef.current) {
          fileInputRef.current.value = ''
        }
      }
    }
  }

  return (
    <div className="relative cursor-pointer group" onClick={handleAvatarClick}>
      <div className="w-20 h-20 rounded-full overflow-hidden border-3 border-white shadow-lg ring-4 ring-white/50 transition-transform group-hover:scale-105">
        {isUploading ? (
          <div className="w-full h-full bg-gray-200 flex items-center justify-center">
            <div className="w-8 h-8 border-3 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
          </div>
        ) : (
          <Avatar className="w-full h-full">
            <AvatarImage
              src={avatarUrl || ''}
              alt="Profile"
              className="object-cover"
            />
            <AvatarFallback className="bg-gradient-to-br from-blue-500 to-purple-600 text-white text-lg font-bold">
              {(name || 'U').charAt(0).toUpperCase()}
            </AvatarFallback>
          </Avatar>
        )}
      </div>
      <div className="absolute bottom-0 right-0 bg-blue-600 rounded-full p-2 shadow-lg border-2 border-white transition-colors group-hover:bg-blue-700">
        <FiCamera className="h-3 w-3 text-white" />
      </div>
      <input
        type="file"
        ref={fileInputRef}
        accept="image/*"
        onChange={handleAvatarUpload}
        style={{ display: 'none' }}
      />
    </div>
  )
}
