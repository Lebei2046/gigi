import type { Message } from '@/store/chatRoomSlice'
import { MessagingUtils } from '@/utils/messaging'

interface FileMessageBubbleProps {
  message: Message
  onDownloadRequest: (
    messageId: string,
    shareCode: string,
    filename: string
  ) => void
}

export default function FileMessageBubble({
  message,
  onDownloadRequest,
}: FileMessageBubbleProps) {
  const handleDownloadClick = () => {
    if (message.shareCode && message.filename) {
      onDownloadRequest(message.id, message.shareCode, message.filename)
    }
  }

  const getFileIcon = (fileType?: string, filename?: string) => {
    if (!fileType && !filename) return '📎'

    const type = fileType?.toLowerCase() || ''
    const ext = filename?.split('.').pop()?.toLowerCase() || ''

    // Images
    if (
      type.startsWith('image/') ||
      ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp'].includes(ext)
    ) {
      return '🖼️'
    }

    // Videos
    if (
      type.startsWith('video/') ||
      ['mp4', 'avi', 'mov', 'mkv', 'webm'].includes(ext)
    ) {
      return '🎬'
    }

    // Audio
    if (
      type.startsWith('audio/') ||
      ['mp3', 'wav', 'flac', 'aac', 'ogg'].includes(ext)
    ) {
      return '🎵'
    }

    // Documents
    if (['pdf', 'doc', 'docx', 'txt', 'rtf'].includes(ext)) {
      return '📄'
    }

    // Archives
    if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext)) {
      return '📦'
    }

    // Code files (like Cargo.toml)
    if (
      [
        'toml',
        'json',
        'js',
        'ts',
        'jsx',
        'tsx',
        'css',
        'html',
        'xml',
        'yaml',
        'yml',
      ].includes(ext)
    ) {
      return '📝'
    }

    // Default
    return '📎'
  }

  const isDownloading =
    message.isDownloading && message.downloadProgress !== undefined
  const isDownloaded =
    !message.isDownloading && message.downloadProgress === 100
  const isDownloadable =
    !message.isOutgoing && message.shareCode && !isDownloading && !isDownloaded

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-2 p-3 bg-gray-50 rounded-lg border hover:bg-gray-100 transition-colors">
        <span className="text-2xl">
          {getFileIcon(message.fileType, message.filename)}
        </span>
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium truncate">{message.filename}</p>
          {message.fileSize && (
            <p className="text-xs text-gray-500">
              {MessagingUtils.formatFileSize(message.fileSize)}
            </p>
          )}
          {isDownloading && (
            <div className="mt-2">
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div
                  className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${message.downloadProgress}%` }}
                />
              </div>
              <p className="text-xs text-blue-600 mt-1">
                Downloading... {message.downloadProgress?.toFixed(1)}%
              </p>
            </div>
          )}
          {isDownloaded && (
            <div className="mt-2">
              <div className="w-full bg-green-200 rounded-full h-2">
                <div
                  className="bg-green-500 h-2 rounded-full"
                  style={{ width: '100%' }}
                />
              </div>
              <p className="text-xs text-green-600 mt-1">Download completed</p>
            </div>
          )}
        </div>
        {isDownloadable && (
          <button
            onClick={handleDownloadClick}
            className="p-2 text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
            title="Download file"
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
                strokeWidth={2}
                d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
              />
            </svg>
          </button>
        )}
        {isDownloaded && (
          <div
            className="p-2 text-green-600 bg-green-50 rounded-lg"
            title="File downloaded"
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
                strokeWidth={2}
                d="M5 13l4 4L19 7"
              />
            </svg>
          </div>
        )}
      </div>
      {message.content && (
        <p className="text-sm break-words text-gray-600 px-1">
          {message.content}
        </p>
      )}
    </div>
  )
}
