import { useAppDispatch, useAppSelector } from '@/store'
import { ask } from '@tauri-apps/plugin-dialog'
import { addLog } from '@/store/logsSlice'
import {
  setShowShareDrawer,
  setSelectedGroup,
  clearChatMessagesAsync,
  removeGroupShareNotification,
  loadChatsAsync,
} from '@/store/chatSlice'
import { clearMessages } from '@/store/chatRoomSlice'
import { MessagingClient } from '@/utils/messaging'
import { getGroup, saveGroup } from '@/utils/chatUtils'
import type { Group, GroupShareMessage, Peer } from '@/utils/messaging'

/**
 * Hook for handling group-related actions
 */
export function useGroupActions() {
  const dispatch = useAppDispatch()
  const selectedGroup = useAppSelector(state => state.chat.selectedGroup)

  const handleShareGroup = (group: Group) => {
    const reduxGroup = {
      ...group,
      createdAt:
        typeof group.createdAt === 'string'
          ? group.createdAt
          : group.createdAt.toISOString(),
    }
    dispatch(setSelectedGroup(reduxGroup))
    dispatch(setShowShareDrawer(true))
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
        joined: true,
        createdAt: new Date(shareMessage.timestamp),
      })

      dispatch(removeGroupShareNotification(shareMessage.from_peer_id))
      dispatch(loadChatsAsync())
    } catch (error) {
      console.error('Failed to accept group share:', error)
    }
  }

  const handleIgnoreGroupShare = (shareMessage: GroupShareMessage) => {
    dispatch(removeGroupShareNotification(shareMessage.from_peer_id))
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

  return {
    handleShareGroup,
    handleSendShareToPeer,
    handleAcceptGroupShare,
    handleIgnoreGroupShare,
    handleClearMessages,
  }
}
