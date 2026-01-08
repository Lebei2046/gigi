import { useEffect } from 'react'
import { useNavigate, useLocation, useParams } from 'react-router-dom'
import { useAppDispatch, useAppSelector } from '@/store'
import { addLog } from '@/store/logsSlice'
import {
  initializeChatRoomAsync,
  loadMessageHistoryAsync,
  initializeChatInfoAsync,
  resetChatRoomState,
} from '@/store/chatRoomSlice'

export function useChatRoomInitialization() {
  const navigate = useNavigate()
  const location = useLocation()
  const { id } = useParams<{ id: string }>()
  const dispatch = useAppDispatch()

  const chatRoomState = useAppSelector(state => state.chatRoom)

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

  // Load message history and initialize chat info when chat room is ready
  useEffect(() => {
    if (
      chatRoomState.isLoading ||
      !chatRoomState.chatId ||
      !chatRoomState.chatName
    )
      return

    dispatch(
      loadMessageHistoryAsync({
        chatId: chatRoomState.chatId,
        isGroupChat: chatRoomState.isGroupChat,
      })
    )

    dispatch(
      initializeChatInfoAsync({
        chatId: chatRoomState.chatId,
        chatName: chatRoomState.chatName,
        isGroupChat: chatRoomState.isGroupChat,
        unreadResetDone: chatRoomState.unreadResetDone,
      })
    )
  }, [
    chatRoomState.isLoading,
    chatRoomState.chatId,
    chatRoomState.chatName,
    chatRoomState.isGroupChat,
    chatRoomState.unreadResetDone,
    dispatch,
  ])

  return chatRoomState
}
