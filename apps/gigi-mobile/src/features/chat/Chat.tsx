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
  clearChatMessages,
} from '@/store/chatSlice'
import { clearMessages } from '@/store/chatRoomSlice'
import { useNavigate, useLocation } from 'react-router-dom'
import { formatShortPeerId } from '@/utils/peerUtils'
import { Trash2 } from 'lucide-react'
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
    const handleUnreadCountReset = (event?: CustomEvent) => {
      if (event?.detail) {
        console.log('📊 Event details:', event.detail)
      }
      loadChats()
      loadGroups()
    }
    window.addEventListener('unreadCountReset', handleUnreadCountReset)

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
              peerId: messageData.from_peer_id,
              lastMessage: `📷 Image: ${messageData.filename}`,
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
              groupId: messageData.group_id,
              lastMessage: messageText,
              timestamp: messageData.timestamp,
            })
          )
        }

        const handleFileMessageReceived = (messageData: any) => {
          // Update latest message for chat list display
          dispatch(
            updateMessage({
              peerId: messageData.from_peer_id,
              lastMessage: `📎 File: ${messageData.filename}`,
              timestamp: messageData.timestamp,
            })
          )
        }

        const handleFileDownloadCompleted = (messageData: any) => {
          // Update the message when download completes
          dispatch(
            updateMessage({
              peerId: messageData.from_nickname,
              lastMessage: `📷 Image: ${messageData.filename}`,
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
    // Convert to Redux-compatible format
    const reduxGroup = {
      ...group,
      createdAt:
        group.createdAt instanceof Date
          ? group.createdAt.toISOString()
          : group.createdAt,
    }
    dispatch(setSelectedGroup(reduxGroup))
    dispatch(setShowShareDrawer(true))
  }

  const handleClearMessages = async (
    chatId: string,
    isGroupChat: boolean,
    chatName: string
  ) => {
    if (!confirm(`Remove messages for ${chatName}`)) {
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
    return (
      <div className="flex items-center justify-center h-full bg-gray-50 p-4">
        <div className="bg-white rounded-2xl shadow-lg border border-gray-100 p-8 max-w-md w-full text-center">
          <div className="w-16 h-16 bg-red-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg
              className="w-8 h-8 text-red-600"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              ></path>
            </svg>
          </div>
          <h3 className="text-xl font-semibold text-gray-900 mb-2">
            Oops! Something went wrong
          </h3>
          <p className="text-red-600 font-medium mb-2">Chat component error</p>
          <p className="text-sm text-gray-600 mb-6 font-mono bg-gray-50 p-3 rounded-lg text-left">
            {String(componentError)}
          </p>
          <button
            onClick={() => window.location.reload()}
            className="w-full py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors duration-200"
          >
            Reload Application
          </button>
        </div>
      </div>
    )
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
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-gray-500">Loading peers...</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200 px-4 py-4">
        <h2 className="text-2xl font-bold text-gray-900">Messages</h2>
      </div>

      <div className="flex-1 overflow-y-auto px-4 py-4">
        {/* Group Share Notifications */}
        {groupShareNotifications.length > 0 && (
          <div className="mb-6 space-y-3">
            {groupShareNotifications.map(notification => (
              <div
                key={notification.from_peer_id}
                className="bg-gradient-to-r from-purple-50 to-pink-50 border border-purple-200 rounded-xl p-4 shadow-sm"
              >
                <div className="flex justify-between items-start">
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-lg">🎉</span>
                      <span className="font-semibold text-purple-800">
                        Group Invitation
                      </span>
                    </div>
                    <div className="text-sm text-gray-700 mb-1">
                      from{' '}
                      <span className="font-medium">
                        {notification.from_nickname}
                      </span>
                    </div>
                    <div className="text-sm font-medium text-purple-600 bg-purple-100 inline-block px-2 py-1 rounded">
                      {notification.group_name}
                    </div>
                  </div>
                </div>
                <div className="flex gap-2 mt-3">
                  <button
                    onClick={() => handleAcceptGroupShare(notification)}
                    className="flex-1 py-2 bg-green-600 hover:bg-green-700 text-white text-sm font-medium rounded-lg transition-colors"
                  >
                    Accept
                  </button>
                  <button
                    onClick={() => handleIgnoreGroupShare(notification)}
                    className="flex-1 py-2 bg-gray-200 hover:bg-gray-300 text-gray-700 text-sm font-medium rounded-lg transition-colors"
                  >
                    Ignore
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Groups Section */}
        {groups.length > 0 && (
          <div className="mb-6">
            <div className="flex items-center gap-2 mb-3">
              <span className="text-lg">👥</span>
              <h3 className="text-lg font-semibold text-gray-900">Groups</h3>
              <span className="bg-blue-100 text-blue-600 text-xs font-medium px-2 py-1 rounded-full">
                {groups.length}
              </span>
              {(() => {
                const totalGroupUnread = groups.reduce((sum, group) => {
                  const chatInfo = chats.find(chat => chat.id === group.id)
                  return sum + (chatInfo?.unreadCount || 0)
                }, 0)
                return (
                  totalGroupUnread > 0 && (
                    <span className="bg-red-500 text-white text-xs font-bold px-2 py-1 rounded-full">
                      {totalGroupUnread}
                    </span>
                  )
                )
              })()}
            </div>
            <div className="space-y-3">
              {groups.map(group => {
                const chatInfo = chats.find(chat => chat.id === group.id)
                const unreadCount = chatInfo?.unreadCount || 0

                return (
                  <div
                    key={group.id}
                    className="bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-md transition-shadow"
                  >
                    <div
                      className="flex justify-between items-start p-4 cursor-pointer"
                      onClick={() => navigate(`/chat/${group.id}`)}
                    >
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-1">
                          <div className="w-10 h-10 bg-blue-100 rounded-full flex items-center justify-center">
                            <span className="text-blue-600 font-semibold">
                              G
                            </span>
                          </div>
                          <div className="flex-1">
                            <div className="flex items-center gap-2">
                              <span className="font-semibold text-gray-900">
                                {group.name}
                              </span>
                              {unreadCount > 0 && (
                                <span className="bg-red-500 text-white text-xs font-bold px-2 py-0.5 rounded-full min-w-[20px] text-center">
                                  {unreadCount}
                                </span>
                              )}
                            </div>
                            <div className="flex items-center gap-2 text-xs text-gray-500">
                              <span className="bg-gray-100 px-2 py-0.5 rounded">
                                {group.joined ? 'Member' : 'Owner'}
                              </span>
                            </div>
                          </div>
                        </div>
                        {latestMessages[group.id] && (
                          <div className="text-sm text-gray-600 mt-2 truncate ml-12">
                            💬 {latestMessages[group.id]}
                          </div>
                        )}
                      </div>
                      <button
                        onClick={e => {
                          e.stopPropagation()
                          handleShareGroup(group)
                        }}
                        className="p-2 text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
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
                            strokeWidth="2"
                            d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m9.032 4.026a3 3 0 10-4.732 2.684m4.732-2.684a3 3 0 00-4.732-2.684"
                          ></path>
                        </svg>
                      </button>
                      <button
                        onClick={e => {
                          e.stopPropagation()
                          handleClearMessages(group.id, true, group.name)
                        }}
                        className="p-2 text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                        title="Clear messages"
                      >
                        <Trash2 className="w-5 h-5" />
                      </button>
                    </div>
                  </div>
                )
              })}
            </div>
          </div>
        )}

        {/* Direct Chats Section */}
        <div>
          <div className="flex items-center gap-2 mb-3">
            <span className="text-lg">💬</span>
            <h3 className="text-lg font-semibold text-gray-900">
              Direct Chats
            </h3>
            <span className="bg-green-100 text-green-600 text-xs font-medium px-2 py-1 rounded-full">
              {peers.length}
            </span>
          </div>
          {peers.length === 0 ? (
            <div className="bg-white border border-gray-200 rounded-xl p-8 text-center">
              <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg
                  className="w-8 h-8 text-gray-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth="2"
                    d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
                  ></path>
                </svg>
              </div>
              <h4 className="text-lg font-semibold text-gray-900 mb-2">
                No Chats Yet
              </h4>
              <p className="text-gray-600 text-sm mb-4">
                Make sure other devices are running Gigi on the same network to
                start chatting.
              </p>
              <div className="bg-blue-50 border border-blue-200 rounded-lg p-3">
                <p className="text-xs text-blue-700">
                  💡 Tip: Both devices need to be connected to the same WiFi
                  network
                </p>
              </div>
            </div>
          ) : (
            <div className="space-y-3">
              {peers.map(peer => {
                const latestMessage = latestMessages[peer.id]
                const chatInfo = chats.find(chat => chat.id === peer.id)
                const unreadCount = chatInfo?.unreadCount || 0

                return (
                  <div
                    key={peer.id}
                    onClick={() => handlePeerClick(peer)}
                    className="bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-md transition-all cursor-pointer hover:border-green-300"
                  >
                    <div className="p-4">
                      <div className="flex justify-between items-start">
                        <div className="flex items-start gap-3 flex-1">
                          <div className="w-12 h-12 bg-gradient-to-br from-green-400 to-green-600 rounded-full flex items-center justify-center flex-shrink-0">
                            <span className="text-white font-bold text-lg">
                              {peer.nickname?.charAt(0).toUpperCase() || '?'}
                            </span>
                          </div>
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2 mb-1">
                              <span className="font-semibold text-gray-900">
                                {peer.nickname}
                              </span>
                              {unreadCount > 0 && (
                                <span className="bg-red-500 text-white text-xs font-bold px-2 py-0.5 rounded-full min-w-[20px] text-center">
                                  {unreadCount}
                                </span>
                              )}
                            </div>
                            <div className="text-xs text-gray-500 font-mono truncate">
                              {formatShortPeerId(peer.id)}
                            </div>
                            {peer.capabilities.length > 0 && (
                              <div className="flex flex-wrap gap-1 mt-1">
                                {peer.capabilities.map((cap, index) => (
                                  <span
                                    key={index}
                                    className="bg-gray-100 text-gray-600 text-xs px-2 py-0.5 rounded"
                                  >
                                    {cap}
                                  </span>
                                ))}
                              </div>
                            )}
                          </div>
                        </div>
                        <div className="text-right ml-3 flex-shrink-0">
                          {chatInfo?.lastMessageTime && (
                            <div className="text-xs text-gray-400 mb-1">
                              {chatInfo.lastMessageTime}
                            </div>
                          )}
                          {(chatInfo?.lastMessage || latestMessage) && (
                            <div className="text-xs text-gray-600 max-w-32 truncate font-medium">
                              {chatInfo?.lastMessage || latestMessage}
                            </div>
                          )}
                        </div>
                        <div className="flex gap-1 mt-2">
                          <button
                            onClick={e => {
                              e.stopPropagation()
                              handleClearMessages(
                                peer.id,
                                false,
                                peer.nickname || 'Unknown'
                              )
                            }}
                            className="p-2 text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                            title="Clear messages"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>
                )
              })}
            </div>
          )}
        </div>

        {/* Share Drawer */}
        {showShareDrawer && selectedGroup && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-end justify-center z-50 animate-fade-in">
            <div className="bg-white w-full max-h-96 rounded-t-3xl shadow-2xl animate-slide-up">
              {/* Handle bar */}
              <div className="flex justify-center pt-2 pb-1">
                <div className="w-12 h-1 bg-gray-300 rounded-full"></div>
              </div>

              <div className="px-6 pb-6">
                <div className="flex justify-between items-center mb-6">
                  <div>
                    <h3 className="text-lg font-semibold text-gray-900">
                      Share Group
                    </h3>
                    <p className="text-sm text-gray-600">
                      "{selectedGroup.name}"
                    </p>
                  </div>
                  <button
                    onClick={() => dispatch(setShowShareDrawer(false))}
                    className="w-10 h-10 bg-gray-100 hover:bg-gray-200 rounded-full flex items-center justify-center transition-colors"
                  >
                    <svg
                      className="w-5 h-5 text-gray-600"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth="2"
                        d="M6 18L18 6M6 6l12 12"
                      ></path>
                    </svg>
                  </button>
                </div>

                {peers.length === 0 ? (
                  <div className="text-center py-8">
                    <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
                      <svg
                        className="w-8 h-8 text-gray-400"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth="2"
                          d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
                        ></path>
                      </svg>
                    </div>
                    <h4 className="text-lg font-semibold text-gray-900 mb-2">
                      No Available Peers
                    </h4>
                    <p className="text-gray-600 text-sm">
                      No peers available to share with
                    </p>
                  </div>
                ) : (
                  <div className="space-y-2 max-h-64 overflow-y-auto">
                    {peers.map(peer => (
                      <div
                        key={peer.id}
                        onClick={() => handleSendShareToPeer(peer)}
                        className="bg-gray-50 hover:bg-gray-100 rounded-xl p-4 cursor-pointer transition-all hover:shadow-md border border-transparent hover:border-gray-200"
                      >
                        <div className="flex items-center gap-3">
                          <div className="w-10 h-10 bg-gradient-to-br from-blue-400 to-blue-600 rounded-full flex items-center justify-center flex-shrink-0">
                            <span className="text-white font-bold">
                              {peer.nickname?.charAt(0).toUpperCase() || '?'}
                            </span>
                          </div>
                          <div className="flex-1 min-w-0">
                            <div className="font-medium text-gray-900">
                              {peer.nickname}
                            </div>
                            <div className="text-xs text-gray-500 font-mono truncate">
                              {formatShortPeerId(peer.id)}
                            </div>
                          </div>
                          <svg
                            className="w-5 h-5 text-gray-400"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                          >
                            <path
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth="2"
                              d="M13 7l5 5m0 0l-5 5m5-5H6"
                            ></path>
                          </svg>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
