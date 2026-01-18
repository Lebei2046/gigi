import { useEffect, useRef } from 'react'
import { useAppDispatch, useAppSelector } from '@/store'
import { loadConversationsAsync, loadGroupsAsync } from '@/store/chatSlice'

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
  const { peers, conversations, groups } = useAppSelector(state => state.chat)
  const lastUnreadCounts = useRef<any[]>([])

  const refreshConversations = async () => {
    try {
      const currentState = await dispatch(loadConversationsAsync()).unwrap()
      const unreadConversations = currentState.filter(
        conv => (conv.unread_count || 0) > 0
      )
      const currentUnreadCounts = unreadConversations.map(conv => ({
        id: conv.id,
        name: conv.name,
        isGroup: conv.is_group,
        unreadCount: conv.unread_count,
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
          (groups, conv) => {
            const key = conv.name.toLowerCase()
            if (!groups[key]) groups[key] = []
            groups[key].push(conv)
            return groups
          },
          {} as Record<string, typeof currentUnreadCounts>
        )

        Object.entries(nameGroups).forEach(([name, convs]) => {
          if (convs.length > 1) {
            console.warn(
              `⚠️ Found ${convs.length} conversation entries with similar name "${name}":`,
              convs
            )
          }
        })

        lastUnreadCounts.current = currentUnreadCounts
      }
    } catch (error) {
      console.error('Failed to load conversations:', error)
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
      refreshConversations()
      refreshGroups()
    }, 3000)

    // Refresh when window gets focus
    const handleFocus = () => {
      refreshConversations()
      refreshGroups()
    }
    window.addEventListener('focus', handleFocus)

    // Refresh when document becomes visible
    const handleVisibilityChange = () => {
      if (!document.hidden) {
        refreshConversations()
        refreshGroups()
      }
    }
    document.addEventListener('visibilitychange', handleVisibilityChange)

    // Refresh on route change
    const handleRouteChange = () => {
      setTimeout(() => {
        refreshConversations()
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
      refreshConversations()
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

  return { refreshConversations, refreshGroups }
}
