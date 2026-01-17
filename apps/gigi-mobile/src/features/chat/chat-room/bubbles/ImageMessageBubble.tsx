import type { Message } from '@/store/chatRoomSlice'
import { useEffect, useState } from 'react'
import { useDispatch } from 'react-redux'
import { loadThumbnailAsync } from '@/store/chatRoomSlice'
import { MessagingClient } from '@/utils/messaging'

interface ImageMessageBubbleProps {
  message: Message
}

export default function ImageMessageBubble({
  message,
}: ImageMessageBubbleProps) {
  const dispatch = useDispatch()
  const [showFullImage, setShowFullImage] = useState(false)
  const [fullImageData, setFullImageData] = useState<string | null>(null)
  const [isLoadingFullImage, setIsLoadingFullImage] = useState(false)
  const [thumbnailData, setThumbnailData] = useState<string | null>(null)

  // Load thumbnail on mount if not available
  useEffect(() => {
    const loadThumbnail = async () => {
      // Check if we already have imageData (for outgoing messages)
      if (message.imageData) {
        setThumbnailData(message.imageData)
        return
      }

      if (message.filePath && !message.thumbnailData && !thumbnailData) {
        // Load thumbnail using file path from backend
        try {
          const data = await MessagingClient.getFileThumbnail(message.filePath)
          if (data && data.length > 0) {
            setThumbnailData(data)
          }
        } catch (error) {
          console.error('Failed to load thumbnail:', error)
        }
      } else if (message.thumbnailData) {
        // Use cached thumbnail from Redux
        setThumbnailData(message.thumbnailData)
      }
    }

    loadThumbnail()
  }, [
    message.shareCode,
    message.thumbnailData,
    message.thumbnailPath,
    message.filePath,
    message.imageData,
    dispatch,
    thumbnailData,
  ])

  const handleImageClick = async () => {
    if (showFullImage) {
      // Close modal and clear full image from memory
      setShowFullImage(false)
      setFullImageData(null)
      return
    }

    if (!fullImageData) {
      // Load full image only when needed, store in component state (not Redux)
      setIsLoadingFullImage(true)
      try {
        let imageData: string
        if (message.filePath) {
          // For received files, use file path (Bob downloads Alice's file)
          imageData = await MessagingClient.getFullImageByPath(message.filePath)
        } else if (message.shareCode) {
          // For sent files, use share code (Alice shares her own file)
          imageData = await MessagingClient.getFullImage(message.shareCode)
        } else {
          throw new Error('No file path or share code available')
        }
        setFullImageData(imageData)
        setShowFullImage(true)
      } catch (error) {
        console.error('Failed to load full image:', error)
      } finally {
        setIsLoadingFullImage(false)
      }
    } else {
      // Already loaded, just show modal
      setShowFullImage(true)
    }
  }

  // Cleanup full image data when unmounting
  useEffect(() => {
    return () => {
      // Clear full image from memory when component unmounts
      setFullImageData(null)
    }
  }, [])

  return (
    <div className="flex flex-col gap-2">
      {/* Display thumbnail, full image data, placeholder, or loading state */}
      {thumbnailData ? (
        <img
          src={thumbnailData}
          alt={message.filename}
          className="max-w-xs max-h-48 rounded-lg object-cover cursor-pointer hover:opacity-90"
          onClick={handleImageClick}
          onError={e => {
            // Clear the invalid image data so placeholder is shown
            e.currentTarget.style.display = 'none'
          }}
        />
      ) : message.shareCode ? (
        // Thumbnail not available, show placeholder for full image
        <div
          onClick={handleImageClick}
          className="w-48 h-48 bg-gray-100 rounded-lg flex items-center justify-center cursor-pointer hover:bg-gray-200 transition-colors border-2 border-dashed border-gray-300"
        >
          <div className="text-center p-4">
            <div className="text-4xl mb-2">🖼️</div>
            <p className="text-sm text-gray-600 break-words max-w-full">
              {message.filename || 'Image'}
            </p>
            <p className="text-xs text-gray-400 mt-1">Click to view</p>
          </div>
        </div>
      ) : (
        // Loading state or no image data
        <div className="w-full h-48 bg-gray-200 animate-pulse rounded-lg" />
      )}

      {/* Full image modal - with loading indicator */}
      {showFullImage && fullImageData && (
        <div
          className="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50"
          onClick={() => {
            setShowFullImage(false)
            setFullImageData(null) // Clear from memory immediately
          }}
        >
          {/* Close/Back button */}
          <button
            onClick={e => {
              e.stopPropagation()
              setShowFullImage(false)
              setFullImageData(null)
            }}
            className="absolute top-4 right-4 z-50 bg-white text-black rounded-full w-10 h-10 flex items-center justify-center shadow-lg hover:bg-gray-200 transition-colors"
            aria-label="Close image"
          >
            ✕
          </button>

          <img
            src={fullImageData}
            alt={message.filename}
            className="max-w-full max-h-full object-contain"
            onClick={e => e.stopPropagation()}
          />
        </div>
      )}

      {/* Full image loading overlay */}
      {isLoadingFullImage && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white p-6 rounded-lg shadow-lg flex flex-col items-center gap-4">
            <div className="w-12 h-12 border-4 border-t-blue-500 border-gray-200 rounded-full animate-spin" />
            <p className="text-gray-700">Loading image...</p>
          </div>
        </div>
      )}

      <p className="text-sm break-words">{message.content}</p>
    </div>
  )
}
