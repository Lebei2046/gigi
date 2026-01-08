import { useEffect, useRef } from 'react'
import { useAppDispatch, useAppSelector } from '@/store'
import { loadChatsAsync, loadGroupsAsync } from '@/store/chatSlice'

/**
 * Hook for handling periodic refreshes of chat data
 * - Polls every 3 seconds
 * - Refreshes on window focus
 * - Refreshes on visibility change
 * - Refreshes on route change
 * - Refreshes on unreadCountReset event
 */
export function useChatDataRefresh() {
  const dispatch = useAppDispatch()
  const { peers, chats, groups } = useAppSelector(state => state.chat)
  const lastUnreadCounts = useRef<any[]>([])

  const refreshChats = async () => {
    try {
      const currentState = await dispatch(loadChatsAsync()).unwrap()
      const unreadChats = currentState.filter(
        chat => (chat.unreadCount || 0) > 0
      )
      const currentUnreadCounts = unreadChats.map(chat => ({
        id: chat.id,
        name: chat.name,
        isGroup: chat.isGroup,
        unreadCount: chat.unreadCount,
      }))

      const lastUnreadCountsStr = JSON.stringify(lastUnreadCounts.current)
      const currentCountsStr = JSON.stringify(currentUnreadCounts)

      if (
        currentUnreadCounts.length > 0 &&
        lastUnreadCountsStr !== currentCountsStr
      ) {
        console.log('🔢 Current unread counts:', currentUnreadCounts)

        // Check for potential duplicates
        const nameGroups = currentUnreadCounts.reduce(
          (groups, chat) => {
            const key = chat.name.toLowerCase()
            if (!groups[key]) groups[key] = []
            groups[key].push(chat)
            return groups
          },
          {} as Record<string, typeof currentUnreadCounts>
        )

        Object.entries(nameGroups).forEach(([name, chats]) => {
          if (chats.length > 1) {
            console.warn(
              `⚠️ Found ${chats.length} chat entries with similar name "${name}":`,
              chats
            )
          }
        })

        lastUnreadCounts.current = currentUnreadCounts
      }
    } catch (error) {
      console.error('Failed to load chats:', error)
    }
  }

  const refreshGroups = async () => {
    try {
      await dispatch(loadGroupsAsync())
    } catch (error) {
      console.error('Failed to load groups:', error)
    }
  }

  useEffect(() => {
    // Set up periodic refresh
    const refreshInterval = setInterval(() => {
      refreshChats()
      refreshGroups()
    }, 3000)

    // Refresh when window gets focus
    const handleFocus = () => {
      refreshChats()
      refreshGroups()
    }
    window.addEventListener('focus', handleFocus)

    // Refresh when document becomes visible
    const handleVisibilityChange = () => {
      if (!document.hidden) {
        refreshChats()
        refreshGroups()
      }
    }
    document.addEventListener('visibilitychange', handleVisibilityChange)

    // Refresh on route change
    const handleRouteChange = () => {
      setTimeout(() => {
        refreshChats()
        refreshGroups()
      }, 50)
    }
    window.addEventListener('popstate', handleRouteChange)

    // Listen for custom unread count reset events
    const handleUnreadCountReset = (event: Event) => {
      const customEvent = event as CustomEvent
      if (customEvent?.detail) {
        console.log('📊 Event details:', customEvent.detail)
      }
      refreshChats()
      refreshGroups()
    }
    window.addEventListener(
      'unreadCountReset',
      handleUnreadCountReset as EventListener
    )

    return () => {
      clearInterval(refreshInterval)
      window.removeEventListener('focus', handleFocus)
      document.removeEventListener('visibilitychange', handleVisibilityChange)
      window.removeEventListener('popstate', handleRouteChange)
      window.removeEventListener('unreadCountReset', handleUnreadCountReset)
    }
  }, [])

  return { refreshChats, refreshGroups }
}
