import { useNavigate } from 'react-router-dom'
import { useAppDispatch } from '@/store'
import { loadChatsAsync } from '@/store/chatSlice'
import { getChatInfo, updateChatInfo } from '@/utils/chatUtils'
import type { Peer } from '@/utils/messaging'

/**
 * Hook for handling peer-related actions
 */
export function usePeerActions() {
  const navigate = useNavigate()
  const dispatch = useAppDispatch()

  const handlePeerClick = async (peer: Peer) => {
    try {
      const existingChat = await getChatInfo(peer.id)
      if (!existingChat) {
        console.log(
          `➕ Creating new chat entry for peer ${peer.id} (${peer.nickname})`
        )
        await updateChatInfo(peer.id, peer.nickname, '', Date.now(), false)
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

  return { handlePeerClick }
}
