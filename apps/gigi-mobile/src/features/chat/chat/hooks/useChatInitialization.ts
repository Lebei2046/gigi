import { useEffect } from 'react'
import { useAppDispatch, useAppSelector } from '@/store'
import { addLog } from '@/store/logsSlice'
import {
  loadPeersAsync,
  loadConversationsAsync,
  loadGroupsAsync,
  setPeers,
  setLoading,
  setError,
  setComponentError,
} from '@/store/chatSlice'
import { ensureConversationsForGroups } from '@/utils/conversationUtils'
import { MessagingClient } from '@/utils/messaging'

/**
 * Hook for initializing chat data - loads peers, chats, and groups
 */
export function useChatInitialization() {
  const dispatch = useAppDispatch()

  // Load peers
  useEffect(() => {
    const loadPeers = async () => {
      try {
        const result = await dispatch(loadPeersAsync()).unwrap()
        dispatch(
          addLog({
            event: 'peers_loaded',
            data: `Loaded ${result.length} connected peers`,
            type: 'info',
          })
        )
      } catch (error) {
        console.error('Failed to load peers:', error)
        dispatch(setError(`Failed to load peers: ${error}`))
        dispatch(setPeers([]))
        dispatch(
          addLog({
            event: 'peers_load_error',
            data: `Failed to load peers: ${error}`,
            type: 'error',
          })
        )
      } finally {
        dispatch(setLoading(false))
      }
    }

    loadPeers()

    // Set up periodic polling as fallback for real-time updates
    const pollInterval = setInterval(async () => {
      try {
        const currentPeers = await MessagingClient.getPeers()
        dispatch(setPeers(currentPeers))
      } catch (error) {
        console.error('Error polling peers:', error)
      }
    }, 3000)

    return () => clearInterval(pollInterval)
  }, [dispatch])

  // Load conversations and groups on mount
  useEffect(() => {
    const loadInitialData = async () => {
      try {
        await dispatch(loadConversationsAsync())
        const groups = await dispatch(loadGroupsAsync()).unwrap()
        // Ensure conversation entries exist for all groups
        await ensureConversationsForGroups(groups)
        // Reload conversations to include newly created group conversations
        await dispatch(loadConversationsAsync())
      } catch (error) {
        console.error('Failed to load initial data:', error)
      }
    }

    loadInitialData()
  }, [dispatch])

  const state = useAppSelector(state => state.chat)

  return {
    peers: state.peers,
    conversations: state.conversations,
    groups: state.groups,
    latestMessages: state.latestMessages,
    groupShareNotifications: state.groupShareNotifications,
    showShareDrawer: state.showShareDrawer,
    selectedGroup: state.selectedGroup,
    loading: state.loading,
    error: state.error,
    componentError: state.componentError,
    loadConversations: () => dispatch(loadConversationsAsync()),
    loadGroups: () => dispatch(loadGroupsAsync()),
  }
}
