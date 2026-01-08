import { useEffect, useRef } from 'react'
import type { Peer, GroupShareMessage } from '@/utils/messaging'
import { MessagingClient, MessagingEvents } from '@/utils/messaging'
import { useAppDispatch, useAppSelector } from '@/store'
import { addLog } from '@/store/logsSlice'
import {
  loadChatsAsync,
  loadGroupsAsync,
  subscribeToGroupsAsync,
  loadPeersAsync,
  clearChatMessagesAsync,
  setPeers,
  addPeer,
  removePeer,
  setLoading,
  setError,
  addGroupShareNotification,
  removeGroupShareNotification,
  setShowShareDrawer,
  setSelectedGroup,
  setComponentError,
  updateDirectMessage,
  updateGroupMessage,
} from '@/store/chatSlice'
import { clearMessages } from '@/store/chatRoomSlice'
import { useNavigate, useLocation } from 'react-router-dom'
import { formatShortPeerId } from '@/utils/peerUtils'
import { ask } from '@tauri-apps/plugin-dialog'
import {
  getGroup,
  saveGroup,
  updateLatestMessage,
  getChatInfo,
  updateChatInfo,
  cleanupInvalidTimestamps,
  ensureChatEntriesForGroups,
  ensureMilliseconds,
} from '@/utils/chatUtils'
import type { Chat, Group } from '@/models/db'
import ChatHeader from './components/chat/ChatHeader'
import GroupShareNotifications from './components/chat/GroupShareNotifications'
import GroupsSection from './components/chat/GroupsSection'
import DirectChatsSection from './components/chat/DirectChatsSection'
import ShareDrawer from './components/chat/ShareDrawer'
import ErrorState from './components/chat/ErrorState'
import LoadingState from './components/chat/LoadingState'

export default function Chat() {
  const dispatch = useAppDispatch()
  const navigate = useNavigate()
  const location = useLocation()
  const lastUnreadCounts = useRef<any[]>([])

  // Select all state from Redux
  const {
    peers,
    loading,
    error,
    chats,
    groups,
    latestMessages,
    groupShareNotifications,
    showShareDrawer,
    selectedGroup,
    componentError,
  } = useAppSelector(state => state.chat)

  // Define loadChats function outside useEffect so it can be used elsewhere
  const loadChats = async () => {
    try {
      dispatch(loadChatsAsync())

      // Get the updated chats from Redux after loading
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

      // Always log to see what's happening (this is key for debugging)
      const lastUnreadCountsStr = JSON.stringify(lastUnreadCounts.current)
      const currentCountsStr = JSON.stringify(currentUnreadCounts)

      if (
        currentUnreadCounts.length > 0 &&
        lastUnreadCountsStr !== currentCountsStr
      ) {
        console.log('🔢 Current unread counts:', currentUnreadCounts)

        // Check for potential duplicates (same name with different IDs)
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

  // Define loadGroups function outside useEffect
  const loadGroups = async () => {
    try {
      dispatch(loadGroupsAsync())
    } catch (error) {
      console.error('Failed to load groups:', error)
    }
  }

  // Handle group share notifications
  useEffect(() => {
    const handleGroupShareReceived = async (
      shareMessage: GroupShareMessage
    ) => {
      // Check if group already exists
      const existingGroup = await getGroup(shareMessage.group_id)
      if (existingGroup) {
        return
      }

      dispatch(addGroupShareNotification(shareMessage))
    }

    MessagingEvents.on('group-share-received', handleGroupShareReceived)

    return () => {
      MessagingEvents.off('group-share-received', handleGroupShareReceived)
    }
  }, [])

  // Subscribe to joined groups on startup
  useEffect(() => {
    dispatch(subscribeToGroupsAsync())
  }, [dispatch])

  // Load chats and groups from IndexedDB and set up periodic refresh
  useEffect(() => {
    // Clean up any existing invalid timestamps first
    cleanupInvalidTimestamps()

    // Ensure all groups have corresponding chat entries
    ensureChatEntriesForGroups()

    // Load immediately
    loadChats()
    loadGroups()

    // Set up periodic refresh to catch updates from ChatRoom
    const refreshInterval = setInterval(() => {
      loadChats()
      loadGroups()
    }, 3000) // Refresh every 3 seconds

    // Also refresh when window gets focus (user returns from ChatRoom)
    const handleFocus = () => {
      loadChats()
      loadGroups()
    }
    window.addEventListener('focus', handleFocus)

    // Also refresh when document becomes visible
    const handleVisibilityChange = () => {
      if (!document.hidden) {
        loadChats()
        loadGroups()
      }
    }
    document.addEventListener('visibilitychange', handleVisibilityChange)

    // Refresh immediately when component mounts (when navigating back from ChatRoom)
    const timer = setTimeout(() => {
      loadChats()
      loadGroups()
    }, 100) // Small delay to ensure navigation is complete

    // Additional refresh on route changes (when coming back from ChatRoom)
    const handleRouteChange = () => {
      setTimeout(() => {
        loadChats()
        loadGroups()
      }, 50)
    }

    // Listen for navigation events
    window.addEventListener('popstate', handleRouteChange)

    // Listen for custom unread count reset events
    const handleUnreadCountReset = (event: Event) => {
      const customEvent = event as CustomEvent
      if (customEvent?.detail) {
        console.log('📊 Event details:', customEvent.detail)
      }
      loadChats()
      loadGroups()
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
      clearTimeout(timer)
    }
  }, [])

  // Refresh data when location changes (navigating back from ChatRoom)
  useEffect(() => {
    loadChats()
    loadGroups()
  }, [location.key]) // location.key changes on navigation

  // Handle peer click to open chat
  const handlePeerClick = async (peer: Peer) => {
    try {
      // Ensure chat entry exists in IndexedDB before navigating
      const existingChat = await getChatInfo(peer.id)
      if (!existingChat) {
        console.log(
          `➕ Creating new chat entry for peer ${peer.id} (${peer.nickname})`
        )
        await updateChatInfo(peer.id, peer.nickname, '', Date.now(), false)
        // Refresh the chats list to include the new entry
        dispatch(loadChatsAsync())
      } else {
        console.log(
          `✅ Chat entry already exists for peer ${peer.id} (${peer.nickname})`
        )
      }
    } catch (error) {
      console.error('Error ensuring chat entry exists:', error)
    }

    navigate(`/chat/${peer.id}`, { state: { peer } })
  }

  useEffect(() => {
    try {
      dispatch(
        addLog({
          event: 'chat_loaded',
          data: 'Chat component loaded successfully',
          type: 'info',
        })
      )

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
          dispatch(setPeers([])) // Ensure we have an empty array on error
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
      }, 3000) // Poll every 3 seconds

      try {
        const handlePeerConnected = (peer: any) => {
          try {
            if (peer && peer.id && peer.nickname) {
              dispatch(addPeer(peer))
              dispatch(
                addLog({
                  event: 'peer_connected',
                  data: `Peer connected: ${peer.nickname} (${peer.id})`,
                  type: 'info',
                })
              )
            }
          } catch (error) {
            console.error('Error in handlePeerConnected:', error)
          }
        }

        const handlePeerDisconnected = (peer: any) => {
          try {
            if (peer && peer.id && peer.nickname) {
              dispatch(removePeer(peer.id))
              dispatch(
                addLog({
                  event: 'peer_disconnected',
                  data: `Peer disconnected: ${peer.nickname} (${peer.id})`,
                  type: 'info',
                })
              )
            }
          } catch (error) {
            console.error('Error in handlePeerDisconnected:', error)
          }
        }

        // Message receiving handler - only handle when NOT in chat room
        const handleMessageReceived = (message: any) => {
          // Don't process if we're currently in a chat room (ChatRoom will handle it)
          const currentPath = window.location.pathname
          if (currentPath.startsWith('/chat/') && currentPath !== '/chat') {
            console.log(
              '🔄 Skipping message processing in Chat component - ChatRoom is active'
            )
            return
          }

          // Save message to localStorage for history
          try {
            const historyKey = `chat_history_${message.from_peer_id}`
            const savedHistory = localStorage.getItem(historyKey)
            let history = savedHistory ? JSON.parse(savedHistory) : []

            // Add the received message to history
            const newMessage = {
              ...message,
              isOutgoing: false, // Received message
            }
            history = [...history, newMessage]
            localStorage.setItem(historyKey, JSON.stringify(history))
          } catch (error) {
            console.error('Failed to save received message to history:', error)
          }

          // Convert timestamp to milliseconds if needed, otherwise use current time
          const timestampMs = message.timestamp
            ? ensureMilliseconds(message.timestamp)
            : Date.now()

          // Update in IndexedDB first - this is the single source of truth
          updateLatestMessage(
            message.from_peer_id,
            message.content,
            timestampMs,
            false,
            false,
            true // Increment unread for incoming direct messages
          )

          // Dispatch Redux action to handle message received
          dispatch(
            updateDirectMessage({
              from_peer_id: message.from_peer_id,
              content: message.content,
              timestamp: timestampMs,
            })
          )

          dispatch(
            addLog({
              event: 'message_received',
              data: `Direct message from ${message.from_nickname}: ${message.content}`,
              type: 'info',
            })
          )
        }

        // Group message receiving handler - only handle when NOT in chat room
        const handleGroupMessageReceived = (message: any) => {
          // Don't process if we're currently in a chat room (ChatRoom will handle it)
          const currentPath = window.location.pathname
          if (currentPath.startsWith('/chat/') && currentPath !== '/chat') {
            console.log(
              '🔄 Skipping group message processing in Chat component - ChatRoom is active'
            )
            return
          }

          // Save message to localStorage for history
          try {
            const historyKey = `chat_history_group_${message.group_id}`
            const savedHistory = localStorage.getItem(historyKey)
            let history = savedHistory ? JSON.parse(savedHistory) : []

            // Add received message to history
            const newMessage = {
              ...message,
              isOutgoing: false, // Received message
              isGroup: true, // Group message
            }
            history = [...history, newMessage]
            localStorage.setItem(historyKey, JSON.stringify(history))
          } catch (error) {
            console.error(
              'Failed to save received group message to history:',
              error
            )
          }

          // Convert timestamp to milliseconds if needed, otherwise use current time
          const timestampMs = message.timestamp
            ? ensureMilliseconds(message.timestamp)
            : Date.now()

          // Update in IndexedDB first - this is single source of truth
          updateLatestMessage(
            message.group_id,
            message.content,
            timestampMs,
            false,
            true,
            true // Increment unread for incoming group messages
          )

          // Dispatch Redux action to handle group message received
          dispatch(
            updateGroupMessage({
              group_id: message.group_id,
              content: message.content,
              timestamp: timestampMs,
            })
          )

          dispatch(
            addLog({
              event: 'group_message_received',
              data: `Group message from ${message.from_nickname} in group ${message.group_id}: ${message.content}`,
              type: 'info',
            })
          )
        }

        // Register event listeners
        MessagingEvents.on('peer-connected', handlePeerConnected)
        MessagingEvents.on('peer-disconnected', handlePeerDisconnected)
        MessagingEvents.on('message-received', handleMessageReceived)
        MessagingEvents.on('group-message', handleGroupMessageReceived)

        // Handle image messages when not in chat room
        const handleImageMessageReceived = (messageData: any) => {
          // Update latest message for chat list display
          dispatch(
            updateDirectMessage({
              from_peer_id: messageData.from_peer_id,
              content: `📷 Image: ${messageData.filename}`,
              timestamp: messageData.timestamp,
            })
          )
        }

        const handleGroupImageMessageReceived = (messageData: any) => {
          // Update latest message for chat list display
          const messageText = messageData.download_error
            ? `❌ Image: ${messageData.filename}`
            : `⬇️ Image: ${messageData.filename}`

          dispatch(
            updateGroupMessage({
              group_id: messageData.group_id,
              content: messageText,
              timestamp: messageData.timestamp,
            })
          )
        }

        const handleFileMessageReceived = (messageData: any) => {
          // Update latest message for chat list display
          dispatch(
            updateDirectMessage({
              from_peer_id: messageData.from_peer_id,
              content: `📎 File: ${messageData.filename}`,
              timestamp: messageData.timestamp,
            })
          )
        }

        const handleFileDownloadCompleted = (messageData: any) => {
          // Update the message when download completes
          dispatch(
            updateDirectMessage({
              from_peer_id: messageData.from_nickname,
              content: `📷 Image: ${messageData.filename}`,
              timestamp: messageData.timestamp,
            })
          )
        }

        MessagingEvents.on('image-message-received', handleImageMessageReceived)
        MessagingEvents.on(
          'group-image-message-received',
          handleGroupImageMessageReceived
        )
        MessagingEvents.on('file-message-received', handleFileMessageReceived)
        MessagingEvents.on(
          'file-download-completed',
          handleFileDownloadCompleted
        )

        // Clean up on unmount
        return () => {
          clearInterval(pollInterval)
          try {
            MessagingEvents.off('peer-connected', handlePeerConnected)
            MessagingEvents.off('peer-disconnected', handlePeerDisconnected)
            MessagingEvents.off('message-received', handleMessageReceived)
            MessagingEvents.off('group-message', handleGroupMessageReceived)
            MessagingEvents.off(
              'image-message-received',
              handleImageMessageReceived
            )
            MessagingEvents.off(
              'group-image-message-received',
              handleGroupImageMessageReceived
            )
            MessagingEvents.off(
              'file-message-received',
              handleFileMessageReceived
            )
            MessagingEvents.off(
              'file-download-completed',
              handleFileDownloadCompleted
            )
            console.log('🧹 Cleaned up Chat event listeners')
          } catch (error) {
            console.error('Error cleaning up event listeners:', error)
          }
        }
      } catch (error) {
        console.error('Error setting up event listeners:', error)
        return () => clearInterval(pollInterval)
      }
    } catch (error) {
      console.error('Chat component useEffect error:', error)
      dispatch(setComponentError(error as Error))
      return () => {}
    }
  }, [dispatch])

  // Group sharing functions
  const handleShareGroup = (group: Group) => {
    const reduxGroup = {
      ...group,
      createdAt:
        typeof group.createdAt === 'string'
          ? group.createdAt
          : group.createdAt.toISOString(), // Convert Date to string for Redux
    }
    dispatch(setSelectedGroup(reduxGroup))
    dispatch(setShowShareDrawer(true))
  }

  const handleClearMessages = async (
    chatId: string,
    isGroupChat: boolean,
    chatName: string
  ) => {
    const confirmed = await ask(`Remove messages for ${chatName}?`, {
      title: 'Confirm',
      kind: 'warning',
    })

    if (!confirmed) {
      return
    }

    try {
      await dispatch(clearChatMessagesAsync({ chatId, isGroupChat })).unwrap()
      dispatch(clearMessages())

      dispatch(
        addLog({
          event: 'messages_cleared',
          data: `Messages cleared for ${isGroupChat ? 'group' : 'direct'} chat: ${chatName}`,
          type: 'info',
        })
      )
    } catch (error) {
      console.error('Failed to clear messages:', error)
      dispatch(
        addLog({
          event: 'messages_clear_failed',
          data: `Failed to clear messages for ${chatName}: ${error}`,
          type: 'error',
        })
      )
    }
  }

  const handleSendShareToPeer = async (targetPeer: Peer) => {
    if (!selectedGroup) return

    try {
      await MessagingClient.sendShareGroupMessage(
        targetPeer.nickname,
        selectedGroup.id,
        selectedGroup.name
      )
      dispatch(setShowShareDrawer(false))
      dispatch(setSelectedGroup(null))
    } catch (error) {
      console.error('Failed to send group share:', error)
    }
  }

  const handleAcceptGroupShare = async (shareMessage: GroupShareMessage) => {
    try {
      await saveGroup({
        id: shareMessage.group_id,
        name: shareMessage.group_name,
        joined: true, // true = invited member who joined the group
        createdAt: new Date(ensureMilliseconds(shareMessage.timestamp)),
      })

      // Remove from notifications
      dispatch(removeGroupShareNotification(shareMessage.from_peer_id))

      // Refresh groups
      dispatch(loadGroupsAsync())
    } catch (error) {
      console.error('Failed to accept group share:', error)
    }
  }

  const handleIgnoreGroupShare = (shareMessage: GroupShareMessage) => {
    // Remove from notifications
    dispatch(removeGroupShareNotification(shareMessage.from_peer_id))
  }

  // Error and loading states
  if (componentError) {
    return <ErrorState error={componentError} />
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-full p-4">
        <p className="text-red-500">Error: {error}</p>
        <button
          onClick={() => window.location.reload()}
          className="mt-4 px-4 py-2 bg-blue-500 text-white rounded"
        >
          Reload
        </button>
      </div>
    )
  }

  if (loading) {
    return <LoadingState />
  }

  return (
    <div className="flex flex-col h-full bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200 px-4 py-4">
        <h2 className="text-2xl font-bold text-gray-900">Messages</h2>
      </div>

      <div className="flex-1 overflow-y-auto px-4 py-4">
        {/* Group Share Notifications */}
        <GroupShareNotifications
          notifications={groupShareNotifications}
          onAccept={handleAcceptGroupShare}
          onIgnore={handleIgnoreGroupShare}
        />

        {/* Groups Section */}
        <GroupsSection
          groups={groups}
          chats={chats}
          latestMessages={latestMessages}
          onGroupClick={groupId => navigate(`/chat/${groupId}`)}
          onShare={handleShareGroup}
          onClearMessages={handleClearMessages}
        />

        {/* Direct Chats Section */}
        <DirectChatsSection
          peers={peers}
          chats={chats}
          latestMessages={latestMessages}
          onPeerClick={handlePeerClick}
          onClearMessages={handleClearMessages}
        />

        {/* Share Drawer */}
        <ShareDrawer
          isOpen={showShareDrawer}
          selectedGroup={selectedGroup}
          peers={peers}
          onClose={() => dispatch(setShowShareDrawer(false))}
          onShare={handleSendShareToPeer}
        />
      </div>
    </div>
  )
}
