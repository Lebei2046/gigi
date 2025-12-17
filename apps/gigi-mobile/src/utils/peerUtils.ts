/**
 * Utility functions for formatting and displaying peer IDs and group IDs
 */

/**
 * Shortens an ID (peer ID or group ID) by showing the first 6 and last 6 characters with "..." in between
 * Example: "12ab34cd56ef78gh90ij" -> "12ab34...78gh90"
 *
 * @param id The full ID string (peer ID or group ID)
 * @returns Shortened ID format, or the original if it's too short
 */
export function formatShortPeerId(id: string): string {
  if (!id) return ''

  // If the ID is 12 characters or shorter, return it as-is
  if (id.length <= 12) {
    return id
  }

  const firstSix = id.substring(0, 6)
  const lastSix = id.substring(id.length - 6)

  return `${firstSix}...${lastSix}`
}

/**
 * Formats a peer/group ID for display in UI components
 * Uses the short format by default
 */
export function formatPeerIdForDisplay(
  id: string,
  useShortFormat: boolean = true
): string {
  if (!id) return ''

  return useShortFormat ? formatShortPeerId(id) : id
}

/**
 * Alias for formatShortPeerId to make it clear it works for group IDs too
 */
export const formatShortGroupId = formatShortPeerId
