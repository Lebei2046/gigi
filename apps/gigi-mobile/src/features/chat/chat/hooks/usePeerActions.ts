import { useNavigate } from 'react-router-dom'
import { useAppDispatch } from '@/store'
import { loadConversationsAsync } from '@/store/chatSlice'
import {
  getConversationInfo,
  updateConversationInfo,
} from '@/utils/conversationUtils'
import type { Peer } from '@/utils/messaging'

/**
 * Hook for handling peer-related actions
 */
export function usePeerActions() {
  const navigate = useNavigate()
  const dispatch = useAppDispatch()

  const handlePeerClick = async (peer: Peer) => {
    try {
      const existingConversation = await getConversationInfo(peer.id)
      if (!existingConversation) {
        console.log(
          `➕ Creating new conversation entry for peer ${peer.id} (${peer.nickname})`
        )
        await updateConversationInfo(
          peer.id,
          peer.nickname,
          '',
          Date.now(),
          false
        )
        dispatch(loadConversationsAsync())
      } else {
        console.log(
          `✅ Conversation entry already exists for peer ${peer.id} (${peer.nickname})`
        )
      }
    } catch (error) {
      console.error('Error ensuring conversation entry exists:', error)
    }

    navigate(`/chat/${peer.id}`, { state: { peer } })
  }

  return { handlePeerClick }
}
