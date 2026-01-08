import { useRef } from 'react'
import { MessagingClient } from '@/utils/messaging'
import { useAppDispatch, useAppSelector } from '@/store'
import { addLog } from '@/store/logsSlice'
import type { Message } from '@/store/chatRoomSlice'
import {
  sendMessageAsync,
  addMessage,
  addImageMessage,
  removeMessage,
  updateMessage,
  updateGroupMessage,
  generateMessageId,
} from '@/store/chatRoomSlice'
import {
  createTextMessage,
  createOutgoingImageMessage,
  createOutgoingFileMessage,
  updateOutgoingImageMessage,
  updateOutgoingFileMessage,
} from '@/utils/messageHelpers'

interface UseMessageActionsParams {
  newMessage: string
  sending: boolean
  isGroupChat: boolean
  peer?: { id: string; nickname: string } | null
  group?: { id: string; name: string } | null
  messages: Message[]
}

export function useMessageActions({
  newMessage,
  sending,
  isGroupChat,
  peer,
  group,
  messages,
}: UseMessageActionsParams) {
  const dispatch = useAppDispatch()
  const sentMessagesRef = useRef<Set<string>>(new Set())

  const handleSendMessage = async () => {
    if (!newMessage.trim() || sending) return
    if (!isGroupChat && !peer) return
    if (isGroupChat && !group) return

    let messageToAdd: Message | null = null
    const timestamp = Date.now()

    try {
      if (isGroupChat) {
        messageToAdd = createTextMessage(
          newMessage.trim(),
          timestamp,
          true,
          true,
          group!
        )
        sentMessagesRef.current.add(messageToAdd.id)
        dispatch(addMessage(messageToAdd))
      } else {
        messageToAdd = createTextMessage(
          newMessage.trim(),
          timestamp,
          true,
          false,
          peer!
        )
        sentMessagesRef.current.add(messageToAdd.id)
        dispatch(addMessage(messageToAdd))
      }

      await dispatch(
        sendMessageAsync({
          content: newMessage.trim(),
          isGroupChat,
          peer,
          group,
        })
      ).unwrap()

      const targetName = isGroupChat ? group!.name : peer!.nickname
      const eventType = isGroupChat ? 'group_message_sent' : 'message_sent'

      dispatch(
        addLog({
          event: eventType,
          data: `${isGroupChat ? 'Group message' : 'Message'} sent to ${targetName}: ${newMessage.trim()}`,
          type: 'info',
        })
      )
    } catch (error) {
      console.error('❌ Failed to send message:', error)

      if (messageToAdd) {
        dispatch(removeMessage(messageToAdd.id))
      }

      const targetName = isGroupChat ? group!.name : peer!.nickname
      const eventType = isGroupChat
        ? 'group_message_send_error'
        : 'message_send_error'

      dispatch(
        addLog({
          event: eventType,
          data: `Failed to send message to ${targetName}: ${error}`,
          type: 'error',
        })
      )
    }
  }

  const handleImageSelect = async () => {
    if (!isGroupChat && !peer) return
    if (isGroupChat && !group) return

    try {
      const filePath = await MessagingClient.selectImageFile()
      if (!filePath) return

      const timestamp = Date.now()
      const filename = filePath.split(/[\\/]/).pop() || 'image'
      const imageMessage = createOutgoingImageMessage(
        filename,
        timestamp,
        isGroupChat
      )

      sentMessagesRef.current.add(imageMessage.id)
      dispatch(addImageMessage(imageMessage))

      const response = isGroupChat
        ? await MessagingClient.sendGroupFileMessageWithPath(
            group!.id,
            filePath
          )
        : await MessagingClient.sendFileMessageWithPath(
            peer!.nickname,
            filePath
          )

      const updatedMessage = updateOutgoingImageMessage(
        imageMessage,
        response.messageId,
        response.imageData,
        filename
      )

      dispatch(
        (isGroupChat ? updateGroupMessage : updateMessage)({
          id: imageMessage.id,
          content: updatedMessage.content,
          imageData: updatedMessage.imageData,
          newId: updatedMessage.id,
        })
      )
    } catch (error) {
      console.error('Failed to send image:', error)
    }
  }

  const handleFileSelect = async () => {
    if (!isGroupChat && !peer) return
    if (isGroupChat && !group) return

    try {
      const filePath = await MessagingClient.selectAnyFile()
      if (!filePath) return

      const timestamp = Date.now()
      const fileInfo = await MessagingClient.getFileInfo(filePath)
      const fileMessage = createOutgoingFileMessage(
        fileInfo,
        timestamp,
        isGroupChat
      )

      sentMessagesRef.current.add(fileMessage.id)
      dispatch(addMessage(fileMessage))

      const response = await MessagingClient.sendFileMessageWithPath(
        peer!.nickname,
        filePath
      )

      const updatedMessage = updateOutgoingFileMessage(
        fileMessage,
        response.messageId,
        fileInfo.name
      )

      dispatch(
        updateMessage({
          id: fileMessage.id,
          content: updatedMessage.content,
          newId: updatedMessage.id,
        })
      )
    } catch (error) {
      console.error('Failed to send file:', error)
    }
  }

  const handleFileDownloadRequest = async (
    messageId: string,
    shareCode: string,
    filename: string
  ) => {
    if (!peer) return

    try {
      const downloadId = await MessagingClient.requestFileFromNickname(
        peer.nickname,
        shareCode
      )

      const updateAction = isGroupChat
        ? updateGroupMessage({
            id: messageId,
            isDownloading: true,
            downloadProgress: 0,
            downloadId: downloadId,
          })
        : updateMessage({
            id: messageId,
            isDownloading: true,
            downloadProgress: 0,
            downloadId: downloadId,
          })

      dispatch(updateAction)
      dispatch(
        addLog({
          event: 'file_download_requested',
          data: `Requested download of ${filename} from ${peer.nickname}`,
          type: 'info',
        })
      )
    } catch (error) {
      console.error('Failed to request file download:', error)

      const resetAction = isGroupChat
        ? updateGroupMessage({
            id: messageId,
            isDownloading: false,
            downloadId: undefined,
          })
        : updateMessage({
            id: messageId,
            isDownloading: false,
            downloadId: undefined,
          })

      dispatch(resetAction)
      dispatch(
        addLog({
          event: 'file_download_request_failed',
          data: `Failed to request download of ${filename}: ${error}`,
          type: 'error',
        })
      )
    }
  }

  return {
    handleSendMessage,
    handleImageSelect,
    handleFileSelect,
    handleFileDownloadRequest,
    sentMessagesRef,
  }
}
