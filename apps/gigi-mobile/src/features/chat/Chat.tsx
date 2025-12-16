import { useEffect, useState } from 'react'
import type { Peer, GroupShareMessage } from '@/utils/messaging'
import { MessagingClient, MessagingEvents } from '@/utils/messaging'
import { useAppDispatch } from '@/store'
import { addLog } from '@/store/logsSlice'
import { useNavigate } from 'react-router-dom'
import {
  getAllChats,
  getAllGroups,
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
    const handleGroupShareReceived = (shareMessage: GroupShareMessage) => {
      console.log('🎉 Received group share:', shareMessage)
      setGroupShareNotifications(prev => [...prev, shareMessage])
    }

    MessagingEvents.on('group-share-received', handleGroupShareReceived)

    return () => {
      MessagingEvents.off('group-share-received', handleGroupShareReceived)
    }
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

  // Error boundary for this component
  try {
    useEffect(() => {
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
              data: `Message from ${message.from_nickname}: ${message.content}`,
              type: 'info',
            })
          )
        }

        // Register event listeners
        MessagingEvents.on('peer-connected', handlePeerConnected)
        MessagingEvents.on('peer-disconnected', handlePeerDisconnected)
        MessagingEvents.on('message-received', handleMessageReceived)
        console.log(
          '✅ Chat.tsx event listeners registered for: peer-connected, peer-disconnected, message-received'
        )

        // Clean up on unmount
        return () => {
          clearInterval(pollInterval)
          try {
            MessagingEvents.off('peer-connected', handlePeerConnected)
            MessagingEvents.off('peer-disconnected', handlePeerDisconnected)
            MessagingEvents.off('message-received', handleMessageReceived)
            console.log('Event listeners cleaned up')
          } catch (error) {
            console.error('Error cleaning up event listeners:', error)
          }
        }
      } catch (error) {
        console.error('Error setting up event listeners:', error)
        return () => clearInterval(pollInterval)
      }
    }, [dispatch, navigate])

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
          joined: true,
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

    return (
      <div className="flex flex-col h-full p-4">
        <h2 className="text-xl font-semibold mb-4">Chats</h2>

        {/* Group Share Notifications */}
        {groupShareNotifications.length > 0 && (
          <div className="mb-4 space-y-2">
            {groupShareNotifications.map(notification => (
              <div
                key={notification.from_peer_id}
                className="p-3 bg-purple-50 border border-purple-200 rounded-lg"
              >
                <div className="flex justify-between items-start">
                  <div className="flex-1">
                    <div className="font-medium text-purple-800">
                      🎉 Group Invitation from {notification.from_nickname}
                    </div>
                    <div className="text-sm text-gray-600">
                      {notification.group_name}
                    </div>
                  </div>
                  <div className="flex space-x-2">
                    <button
                      onClick={() => handleAcceptGroupShare(notification)}
                      className="px-3 py-1 bg-green-500 text-white text-xs rounded hover:bg-green-600"
                    >
                      Accept
                    </button>
                    <button
                      onClick={() => handleIgnoreGroupShare(notification)}
                      className="px-3 py-1 bg-gray-500 text-white text-xs rounded hover:bg-gray-600"
                    >
                      Ignore
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Groups Section */}
        {groups.length > 0 && (
          <div className="mb-6">
            <h3 className="text-lg font-medium mb-3">Groups</h3>
            <div className="space-y-2">
              {groups.map(group => (
                <div
                  key={group.id}
                  className="p-3 bg-blue-50 border border-blue-200 rounded-lg"
                >
                  <div className="flex justify-between items-start">
                    <div className="flex-1">
                      <div className="font-medium text-blue-800">
                        👥 {group.name}
                      </div>
                      <div className="text-xs text-gray-500">
                        {group.joined ? 'Joined' : 'Not joined'} • {group.id}
                      </div>
                    </div>
                    <button
                      onClick={() => handleShareGroup(group)}
                      className="px-3 py-1 bg-blue-500 text-white text-xs rounded hover:bg-blue-600"
                    >
                      Share
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Direct Chats Section */}
        <div>
          <h3 className="text-lg font-medium mb-3">Direct Chats</h3>
          {peers.length === 0 ? (
            <div className="flex items-center justify-center h-full">
              <p className="text-gray-500">
                No peers found. Make sure other devices are running Gigi on the
                same network.
              </p>
            </div>
          ) : (
            <div className="space-y-2">
              {peers.map(peer => {
                const latestMessage = latestMessages[peer.id]
                const chatInfo = chats.find(chat => chat.id === peer.id)

                return (
                  <div
                    key={peer.id}
                    onClick={() => handlePeerClick(peer)}
                    className="p-3 bg-green-50 border border-green-200 rounded-lg cursor-pointer hover:bg-green-100 transition-colors"
                  >
                    <div className="flex justify-between items-start">
                      <div className="flex-1">
                        <div className="font-medium text-green-800">
                          {peer.nickname}
                        </div>
                        <div className="text-sm text-gray-600">{peer.id}</div>
                        {peer.capabilities.length > 0 && (
                          <div className="text-xs text-gray-500 mt-1">
                            {peer.capabilities.join(', ')}
                          </div>
                        )}
                      </div>
                      {(chatInfo?.lastMessage || latestMessage) && (
                        <div className="ml-2 text-right">
                          <div className="text-xs text-blue-600 font-medium">
                            💬{' '}
                            {chatInfo?.unreadCount && chatInfo.unreadCount > 0
                              ? `(${chatInfo.unreadCount})`
                              : 'Latest'}
                          </div>
                          <div className="text-xs text-gray-600 max-w-32 truncate font-medium">
                            {chatInfo?.lastMessage || latestMessage}
                          </div>
                          {chatInfo?.lastMessageTime && (
                            <div className="text-xs text-gray-400">
                              {chatInfo.lastMessageTime}
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  </div>
                )
              })}
            </div>
          )}
        </div>

        {/* Share Drawer */}
        {showShareDrawer && selectedGroup && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-end justify-center z-50">
            <div className="bg-white w-full max-h-96 rounded-t-2xl p-4">
              <div className="flex justify-between items-center mb-4">
                <h3 className="text-lg font-semibold">
                  Share "{selectedGroup.name}" with:
                </h3>
                <button
                  onClick={() => setShowShareDrawer(false)}
                  className="text-gray-500 hover:text-gray-700"
                >
                  ✕
                </button>
              </div>
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {peers.map(peer => (
                  <div
                    key={peer.id}
                    onClick={() => handleSendShareToPeer(peer)}
                    className="p-3 bg-gray-50 rounded-lg cursor-pointer hover:bg-gray-100 transition-colors"
                  >
                    <div className="font-medium">{peer.nickname}</div>
                    <div className="text-sm text-gray-600">{peer.id}</div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>
    )
  } catch (error) {
    console.error('Chat component error:', error)
    return (
      <div className="flex items-center justify-center h-full p-4">
        <div className="text-center">
          <p className="text-red-500">Chat component error</p>
          <p className="text-sm text-gray-500">{String(error)}</p>
          <button
            onClick={() => window.location.reload()}
            className="mt-4 px-4 py-2 bg-blue-500 text-white rounded"
          >
            Reload
          </button>
        </div>
      </div>
    )
  }
}
