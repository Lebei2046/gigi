import { useEffect, useState } from 'react'
import type { Peer, GroupShareMessage } from '@/utils/messaging'
import { MessagingClient, MessagingEvents } from '@/utils/messaging'
import { useAppDispatch } from '@/store'
import { addLog } from '@/store/logsSlice'
import { useNavigate } from 'react-router-dom'
import { formatShortPeerId } from '@/utils/peerUtils'
import {
  getAllChats,
  getAllGroups,
  getGroup,
  saveGroup,
  updateLatestMessage,
  getChatInfo,
  updateChatInfo,
  cleanupInvalidTimestamps,
} from '@/utils/chatUtils'
import type { Chat, Group } from '@/models/db'

export default function Chat() {
  const [peers, setPeers] = useState<Peer[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [chats, setChats] = useState<Chat[]>([])
  const [groups, setGroups] = useState<Group[]>([])
  const [latestMessages, setLatestMessages] = useState<Record<string, string>>(
    {}
  )
  const [groupShareNotifications, setGroupShareNotifications] = useState<
    GroupShareMessage[]
  >([])
  const [showShareDrawer, setShowShareDrawer] = useState(false)
  const [selectedGroup, setSelectedGroup] = useState<Group | null>(null)
  const [componentError, setComponentError] = useState<Error | null>(null)
  const dispatch = useAppDispatch()
  const navigate = useNavigate()

  // Define loadChats function outside useEffect so it can be used elsewhere
  const loadChats = async () => {
    try {
      console.log('🔄 Loading chats from IndexedDB...')
      const allChats = await getAllChats()
      setChats(allChats)

      // Update latestMessages from chats - this is the single source of truth
      const messagesFromChats: Record<string, string> = {}
      allChats.forEach(chat => {
        if (chat.lastMessage) {
          messagesFromChats[chat.id] = chat.lastMessage
        }
      })
      setLatestMessages(messagesFromChats)

      console.log('📚 Loaded chats from IndexedDB:', allChats)
      console.log('💬 Latest messages from IndexedDB:', messagesFromChats)
    } catch (error) {
      console.error('Failed to load chats:', error)
    }
  }

  // Define loadGroups function outside useEffect
  const loadGroups = async () => {
    try {
      console.log('🔄 Loading groups from IndexedDB...')
      const allGroups = await getAllGroups()
      setGroups(allGroups)
      console.log('👥 Loaded groups from IndexedDB:', allGroups)
    } catch (error) {
      console.error('Failed to load groups:', error)
    }
  }

  // Handle group share notifications
  useEffect(() => {
    const handleGroupShareReceived = async (
      shareMessage: GroupShareMessage
    ) => {
      console.log('🎉 Received group share:', shareMessage)

      // Check if group already exists
      const existingGroup = await getGroup(shareMessage.group_id)
      if (existingGroup) {
        console.log('🔄 Group already exists, ignoring share message')
        return
      }

      setGroupShareNotifications(prev => [...prev, shareMessage])
    }

    MessagingEvents.on('group-share-received', handleGroupShareReceived)

    return () => {
      MessagingEvents.off('group-share-received', handleGroupShareReceived)
    }
  }, [])

  // Subscribe to joined groups on startup
  useEffect(() => {
    const subscribeToGroups = async () => {
      try {
        const allGroups = await getAllGroups()
        const joinedGroups = allGroups.filter(group => group.joined)

        console.log(
          '🎯 Chat.tsx: Found groups to subscribe to:',
          joinedGroups.length
        )
        console.log(
          '🎯 Chat.tsx: Joined groups:',
          joinedGroups.map(g => ({ id: g.id, name: g.name }))
        )

        for (const group of joinedGroups) {
          try {
            console.log(
              '🔔 Chat.tsx: Attempting to join group:',
              group.id,
              '(',
              group.name,
              ')'
            )
            await MessagingClient.joinGroup(group.id)
            console.log(
              '✅ Chat.tsx: Successfully joined group topic:',
              group.name,
              'ID:',
              group.id
            )
          } catch (error) {
            console.error(
              `❌ Chat.tsx: Failed to subscribe to group ${group.name}:`,
              error
            )
          }
        }
      } catch (error) {
        console.error('❌ Chat.tsx: Failed to subscribe to groups:', error)
      }
    }

    subscribeToGroups()
  }, [])

  // Load chats and groups from IndexedDB and set up periodic refresh
  useEffect(() => {
    // Clean up any existing invalid timestamps first
    cleanupInvalidTimestamps()

    // Load immediately
    loadChats()
    loadGroups()

    // Set up more frequent refresh to catch updates from ChatRoom
    const refreshInterval = setInterval(() => {
      loadChats()
      loadGroups()
    }, 1000) // Refresh every 1 second

    // Also refresh when window gets focus (user returns from ChatRoom)
    const handleFocus = () => {
      console.log('🔄 Window focused, refreshing chats and groups...')
      loadChats()
      loadGroups()
    }
    window.addEventListener('focus', handleFocus)

    // Also refresh when document becomes visible
    const handleVisibilityChange = () => {
      if (!document.hidden) {
        console.log('🔄 Document visible, refreshing chats and groups...')
        loadChats()
        loadGroups()
      }
    }
    document.addEventListener('visibilitychange', handleVisibilityChange)

    return () => {
      clearInterval(refreshInterval)
      window.removeEventListener('focus', handleFocus)
      document.removeEventListener('visibilitychange', handleVisibilityChange)
    }
  }, [])

  // Handle peer click to open chat
  const handlePeerClick = async (peer: Peer) => {
    try {
      // Ensure chat entry exists in IndexedDB before navigating
      const existingChat = await getChatInfo(peer.id)
      if (!existingChat) {
        await updateChatInfo(peer.id, peer.nickname, '', Date.now(), false)
        console.log(
          '📝 Created chat entry for peer before navigation:',
          peer.id
        )
        // Refresh the chats list to include the new entry
        loadChats()
      }
    } catch (error) {
      console.error('Error ensuring chat entry exists:', error)
    }

    navigate(`/chat/${peer.id}`, { state: { peer } })
  }

  useEffect(() => {
    try {
      console.log('Chat component mounting...')
      console.log(
        '🎯 Chat.tsx useEffect running, dispatch:',
        dispatch,
        'navigate:',
        navigate
      )
      dispatch(
        addLog({
          event: 'chat_loaded',
          data: 'Chat component loaded successfully',
          type: 'info',
        })
      )

      const loadPeers = async () => {
        try {
          console.log('Loading real peers...')

          // Add timeout to prevent hanging
          const timeoutPromise = new Promise((_, reject) => {
            setTimeout(() => reject(new Error('Timeout loading peers')), 5000)
          })

          const peersPromise = MessagingClient.getPeers()
          const connectedPeers = (await Promise.race([
            peersPromise,
            timeoutPromise,
          ])) as Peer[]

          console.log('Connected peers:', connectedPeers)
          setPeers(connectedPeers)
          dispatch(
            addLog({
              event: 'peers_loaded',
              data: `Loaded ${connectedPeers.length} connected peers`,
              type: 'info',
            })
          )
        } catch (error) {
          console.error('Failed to load peers:', error)
          setError(`Failed to load peers: ${error}`)
          setPeers([]) // Ensure we have an empty array on error
          dispatch(
            addLog({
              event: 'peers_load_error',
              data: `Failed to load peers: ${error}`,
              type: 'error',
            })
          )
        } finally {
          setLoading(false)
        }
      }

      loadPeers()

      // Set up periodic polling as fallback for real-time updates
      const pollInterval = setInterval(async () => {
        try {
          console.log('Polling for peers...')
          const currentPeers = await MessagingClient.getPeers()
          console.log('Current peers from poll:', currentPeers)
          setPeers(currentPeers)
        } catch (error) {
          console.error('Error polling peers:', error)
        }
      }, 3000) // Poll every 3 seconds

      // Also try event listeners with debugging
      try {
        console.log('Setting up event listeners...')

        const handlePeerConnected = (peer: any) => {
          console.log('🔗 Peer connected event received:', peer)
          try {
            if (peer && peer.id && peer.nickname) {
              setPeers(prev => {
                const exists = prev.find(p => p.id === peer.id)
                if (!exists) {
                  console.log('Adding new peer to state:', peer)
                  return [...prev, peer]
                }
                return prev
              })
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
          console.log('❌ Peer disconnected event received:', peer)
          try {
            if (peer && peer.id && peer.nickname) {
              setPeers(prev => prev.filter(p => p.id !== peer.id))
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

        // Message receiving handler
        const handleMessageReceived = (message: any) => {
          console.log('📨 Message received in Chat.tsx:', message)
          console.log('📨 From peer ID:', message.from_peer_id)
          console.log('📨 Content:', message.content)

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
            console.log(
              '💾 Saved received message to history for peer:',
              message.from_peer_id
            )
          } catch (error) {
            console.error('Failed to save received message to history:', error)
          }

          // Convert timestamp to milliseconds if needed, otherwise use current time
          const timestampMs =
            message.timestamp && message.timestamp < 1000000000000
              ? message.timestamp * 1000
              : message.timestamp || Date.now()

          // Update in IndexedDB first - this is the single source of truth
          updateLatestMessage(
            message.from_peer_id,
            message.content,
            timestampMs,
            false,
            false
          )

          // Then update local state to match IndexedDB
          setLatestMessages(prev => {
            const newMessages = {
              ...prev,
              [message.from_peer_id]: message.content,
            }
            console.log('📨 Updated latestMessages:', newMessages)
            return newMessages
          })

          dispatch(
            addLog({
              event: 'message_received',
              data: `Direct message from ${message.from_nickname}: ${message.content}`,
              type: 'info',
            })
          )
        }

        // Group message receiving handler
        const handleGroupMessageReceived = (message: any) => {
          console.log('🔥 GROUP MESSAGE RECEIVED in Chat.tsx:', message)
          console.log('🔥 From peer ID:', message.from_peer_id)
          console.log('🔥 Group ID:', message.group_id)
          console.log('🔥 From nickname:', message.from_nickname)
          console.log('🔥 Content:', message.content)
          console.log('🔥 Timestamp:', message.timestamp)

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
            console.log(
              '💾 Saved received group message to history for group:',
              message.group_id
            )
          } catch (error) {
            console.error(
              'Failed to save received group message to history:',
              error
            )
          }

          // Convert timestamp to milliseconds if needed, otherwise use current time
          const timestampMs =
            message.timestamp && message.timestamp < 1000000000000
              ? message.timestamp * 1000
              : message.timestamp || Date.now()

          // Update in IndexedDB first - this is single source of truth
          updateLatestMessage(
            message.group_id,
            message.content,
            timestampMs,
            false,
            true
          )

          // Then update local state to match IndexedDB
          setLatestMessages(prev => {
            const newMessages = {
              ...prev,
              [message.group_id]: message.content,
            }
            console.log('💬 Updated latest messages (group):', newMessages)
            return newMessages
          })

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
        console.log(
          '🎯 Chat.tsx event listeners registered for: peer-connected, peer-disconnected, message-received, group-message'
        )
        console.log('🎯 Listening for group messages on all groups...')

        // Clean up on unmount
        return () => {
          clearInterval(pollInterval)
          try {
            MessagingEvents.off('peer-connected', handlePeerConnected)
            MessagingEvents.off('peer-disconnected', handlePeerDisconnected)
            MessagingEvents.off('message-received', handleMessageReceived)
            MessagingEvents.off('group-message', handleGroupMessageReceived)
            console.log('Event listeners cleaned up')
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
      setComponentError(error as Error)
      return () => {}
    }
  }, [dispatch, navigate])

  // Group sharing functions
  const handleShareGroup = (group: Group) => {
    setSelectedGroup(group)
    setShowShareDrawer(true)
  }

  const handleSendShareToPeer = async (targetPeer: Peer) => {
    if (!selectedGroup) return

    try {
      await MessagingClient.sendShareGroupMessage(
        targetPeer.nickname,
        selectedGroup.id,
        selectedGroup.name
      )
      console.log('📤 Group share sent successfully')
      setShowShareDrawer(false)
      setSelectedGroup(null)
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
        createdAt: new Date(shareMessage.timestamp * 1000),
      })

      // Remove from notifications
      setGroupShareNotifications(prev =>
        prev.filter(msg => msg.from_peer_id !== shareMessage.from_peer_id)
      )

      // Refresh groups
      loadGroups()

      console.log('✅ Group share accepted and saved')
    } catch (error) {
      console.error('Failed to accept group share:', error)
    }
  }

  const handleIgnoreGroupShare = (shareMessage: GroupShareMessage) => {
    // Remove from notifications
    setGroupShareNotifications(prev =>
      prev.filter(msg => msg.from_peer_id !== shareMessage.from_peer_id)
    )
    console.log('🚫 Group share ignored')
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

  console.log(
    '🎨 Chat.tsx rendering with peers:',
    peers.length,
    'latestMessages:',
    latestMessages,
    'chats:',
    chats
  )

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
            </div>
            <div className="space-y-3">
              {groups.map(group => (
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
                          <span className="text-blue-600 font-semibold">G</span>
                        </div>
                        <div>
                          <div className="font-semibold text-gray-900">
                            {group.name}
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
                  </div>
                </div>
              ))}
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
                    onClick={() => setShowShareDrawer(false)}
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
