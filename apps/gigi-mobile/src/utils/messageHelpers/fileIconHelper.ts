/**
 * Get appropriate file icon based on file type or extension
 */
export function getFileIcon(fileType?: string, filename?: string): string {
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

  // Code files
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
