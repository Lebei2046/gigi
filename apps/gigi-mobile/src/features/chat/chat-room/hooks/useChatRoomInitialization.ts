import { useEffect, useRef } from 'react'
import { useNavigate, useLocation, useParams } from 'react-router-dom'
import { useAppDispatch, useAppSelector } from '@/store'
import { addLog } from '@/store/logsSlice'
import {
  initializeChatRoomAsync,
  loadMessageHistoryAsync,
  initializeChatInfoAsync,
  resetChatRoomState,
  loadMessagesFromBackendAsync,
} from '@/store/chatRoomSlice'

export function useChatRoomInitialization() {
  const navigate = useNavigate()
  const location = useLocation()
  const { id } = useParams<{ id: string }>()
  const dispatch = useAppDispatch()

  const chatRoomState = useAppSelector(state => state.chatRoom)
  const messagesLoadedRef = useRef(false)

  // Initialize chat room
  useEffect(() => {
    if (!id) {
      console.log('❌ ChatRoom: No id provided, navigating to /chat')
      navigate('/chat')
      return
    }

    console.log(
      '🚀 ChatRoom: Initializing with id:',
      id,
      'peer:',
      location.state?.peer
    )
    dispatch(initializeChatRoomAsync({ id, peer: location.state?.peer }))
      .unwrap()
      .then(result => {
        console.log('✅ ChatRoom: Initialization successful:', result)
        if (result.isGroupChat && result.group) {
          dispatch(
            addLog({
              event: 'group_chat_room_opened',
              data: `Opened group chat: ${result.group.name}`,
              type: 'info',
            })
          )
        } else if (!result.isGroupChat && result.peer) {
          dispatch(
            addLog({
              event: 'chat_room_opened',
              data: `Opened chat with ${result.peer.nickname}`,
              type: 'info',
            })
          )
        } else {
          console.error(
            '⚠️ ChatRoom: Invalid result - neither group nor peer:',
            result
          )
          navigate('/chat')
        }
      })
      .catch(error => {
        console.error('❌ Failed to initialize chat room:', error)
        console.error('Error details:', {
          id,
          peer: location.state?.peer,
          errorMessage: error.message || error,
          errorStack: error.stack,
        })

        dispatch(resetChatRoomState())
        navigate('/chat')
      })
  }, [id, navigate, dispatch])

  // Load message history from backend and initialize chat info when chat room is ready
  useEffect(() => {
    if (
      chatRoomState.isLoading ||
      !chatRoomState.chatId ||
      !chatRoomState.chatName
    )
      return

    // Prevent re-dispatching if messages have already been loaded for this chat room
    if (messagesLoadedRef.current) {
      return
    }

    // Load messages from backend (new unified storage approach)
    // For both peer chats and group chats, use chatId (full peer ID or group ID)
    const peerId = chatRoomState.chatId

    console.log('📥 Loading messages from backend:', {
      isGroupChat: chatRoomState.isGroupChat,
      peerId,
      chatId: chatRoomState.chatId,
      chatName: chatRoomState.chatName,
    })

    console.log('🔄 Dispatching loadMessagesFromBackendAsync...')
    dispatch(
      loadMessagesFromBackendAsync({
        peerId,
        limit: 50,
        offset: 0,
      })
    )

    // Still initialize chat info (reset unread counts, etc.)
    dispatch(
      initializeChatInfoAsync({
        chatId: chatRoomState.chatId,
        chatName: chatRoomState.chatName,
        isGroupChat: chatRoomState.isGroupChat,
        unreadResetDone: chatRoomState.unreadResetDone,
      })
    )

    // Mark messages as loaded
    messagesLoadedRef.current = true
  }, [
    chatRoomState.isLoading,
    chatRoomState.chatId,
    chatRoomState.chatName,
    chatRoomState.isGroupChat,
    chatRoomState.unreadResetDone,
    dispatch,
  ])

  // Reset the ref when chat room changes OR when navigating (location.key changes)
  useEffect(() => {
    messagesLoadedRef.current = false
  }, [id, location.key])

  return chatRoomState
}
