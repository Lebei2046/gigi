import React, { useState, useEffect, useRef } from 'react'
import { useAppDispatch } from '@/store'
import { addLog } from '@/store/logsSlice'
import { agentMessagingClient } from '@/utils/agentMessaging'
import type { Agent } from '@/utils/agentMessaging'
import type { TextMessage, FileMessage } from '@gigi/amp-ts'
import { MessagingClient } from '@/utils/messaging'
import {
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Send, Paperclip, ArrowLeft } from 'lucide-react'

interface AgentChatProps {
  agent: Agent
  onBack: () => void
}

interface ChatMessage {
  id: string
  content: string
  sender: 'user' | 'agent'
  timestamp: number
  type: 'text' | 'file'
  filename?: string
  fileSize?: number
  shareCode?: string
}

const AgentChat: React.FC<AgentChatProps> = ({ agent, onBack }) => {
  const dispatch = useAppDispatch()
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [newMessage, setNewMessage] = useState('')
  const [sending, setSending] = useState(false)
  const messageListRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    // Register message handlers for this agent
    const handleTextMessage = (message: TextMessage) => {
      if (message.sender.id === agent.id || message.sender.type === 'agent') {
        const chatMessage: ChatMessage = {
          id: `${message.timestamp}-${message.sender.id}`,
          content: message.content,
          sender: 'agent',
          timestamp: message.timestamp,
          type: 'text',
        }
        setMessages(prev => [...prev, chatMessage])
      }
    }

    const handleFileMessage = (message: FileMessage) => {
      if (message.sender.id === agent.id || message.sender.type === 'agent') {
        const chatMessage: ChatMessage = {
          id: `${message.timestamp}-${message.sender.id}`,
          content: message.filename || 'File',
          sender: 'agent',
          timestamp: message.timestamp,
          type: 'file',
          filename: message.filename,
          fileSize: message.fileSize,
          shareCode: message.shareCode || '',
        }
        setMessages(prev => [...prev, chatMessage])
      }
    }

    agentMessagingClient.registerMessageHandler('text', handleTextMessage)
    agentMessagingClient.registerMessageHandler('file', handleFileMessage)

    return () => {
      // Cleanup handlers
    }
  }, [agent.id])

  useEffect(() => {
    // Scroll to bottom when messages change
    if (messageListRef.current) {
      messageListRef.current.scrollTop = messageListRef.current.scrollHeight
    }
  }, [messages])

  const handleSendMessage = async () => {
    if (!newMessage.trim() || sending) return

    try {
      setSending(true)

      // Add message to local state
      const chatMessage: ChatMessage = {
        id: `${Date.now()}-user`,
        content: newMessage.trim(),
        sender: 'user',
        timestamp: Date.now(),
        type: 'text',
      }
      setMessages(prev => [...prev, chatMessage])

      // Send message to agent
      await agentMessagingClient.sendTextMessage(
        newMessage.trim(),
        'specific',
        [agent.id]
      )

      dispatch(
        addLog({
          event: 'agent_message_sent',
          data: `Sent message to agent ${agent.name}: ${newMessage.trim()}`,
          type: 'info',
        })
      )

      setNewMessage('')
    } catch (error) {
      console.error('Failed to send message:', error)
      dispatch(
        addLog({
          event: 'agent_message_send_error',
          data: `Failed to send message to agent ${agent.name}: ${error}`,
          type: 'error',
        })
      )
    } finally {
      setSending(false)
    }
  }

  const handleFileSelect = async () => {
    try {
      const filePath = await MessagingClient.selectAnyFile()
      if (!filePath) return

      // Share file and get share code
      const shareCode = await MessagingClient.shareFile(filePath)

      // Get file info
      const fileInfo = await MessagingClient.getFileInfo(filePath)

      // Add file message to local state
      const chatMessage: ChatMessage = {
        id: `${Date.now()}-user-file`,
        content: fileInfo.name,
        sender: 'user',
        timestamp: Date.now(),
        type: 'file',
        filename: fileInfo.name,
        fileSize: fileInfo.size,
        shareCode,
      }
      setMessages(prev => [...prev, chatMessage])

      // Send file message to agent
      await agentMessagingClient.sendFileMessage(
        shareCode,
        fileInfo.name,
        fileInfo.size,
        'specific',
        [agent.id]
      )

      dispatch(
        addLog({
          event: 'agent_file_sent',
          data: `Sent file to agent ${agent.name}: ${fileInfo.name}`,
          type: 'info',
        })
      )
    } catch (error) {
      console.error('Failed to send file:', error)
      dispatch(
        addLog({
          event: 'agent_file_send_error',
          data: `Failed to send file to agent ${agent.name}: ${error}`,
          type: 'error',
        })
      )
    }
  }

  const handleFileDownload = async (shareCode: string, filename: string) => {
    try {
      // Request file download
      await MessagingClient.requestFileFromNickname(agent.name, shareCode)

      dispatch(
        addLog({
          event: 'agent_file_download_requested',
          data: `Requested download of ${filename} from agent ${agent.name}`,
          type: 'info',
        })
      )
    } catch (error) {
      console.error('Failed to request file download:', error)
      dispatch(
        addLog({
          event: 'agent_file_download_error',
          data: `Failed to download file from agent ${agent.name}: ${error}`,
          type: 'error',
        })
      )
    }
  }

  return (
    <div className="flex flex-col h-full">
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={onBack}
            className="h-8 w-8"
          >
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <div>
            <CardTitle className="text-lg font-semibold">
              {agent.name}
            </CardTitle>
            <CardDescription>
              {agent.type} v{agent.version}
            </CardDescription>
          </div>
        </div>
      </CardHeader>

      <ScrollArea className="flex-1 px-4">
        <div ref={messageListRef} className="space-y-4 py-4">
          {messages.map(message => (
            <div
              key={message.id}
              className={`flex ${message.sender === 'user' ? 'justify-end' : 'justify-start'}`}
            >
              <div
                className={`max-w-[80%] rounded-lg px-4 py-2 ${message.sender === 'user' ? 'bg-primary text-primary-foreground' : 'bg-muted'}`}
              >
                {message.type === 'text' ? (
                  <p>{message.content}</p>
                ) : (
                  <div className="flex flex-col">
                    <p className="font-medium">{message.filename}</p>
                    {message.fileSize && (
                      <p className="text-xs text-muted-foreground">
                        {formatFileSize(message.fileSize)}
                      </p>
                    )}
                    {message.sender === 'agent' && message.shareCode && (
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() =>
                          handleFileDownload(
                            message.shareCode!,
                            message.filename!
                          )
                        }
                        className="mt-1 w-full justify-center"
                      >
                        Download
                      </Button>
                    )}
                  </div>
                )}
                <p className="text-xs text-right mt-1 text-muted-foreground">
                  {new Date(message.timestamp).toLocaleTimeString()}
                </p>
              </div>
            </div>
          ))}
        </div>
      </ScrollArea>

      <CardFooter className="px-4 pb-4">
        <div className="flex w-full gap-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={handleFileSelect}
            className="h-10 w-10"
          >
            <Paperclip className="h-5 w-5" />
          </Button>
          <Textarea
            value={newMessage}
            onChange={e => setNewMessage(e.target.value)}
            placeholder={`Message ${agent.name}...`}
            className="flex-1 resize-none"
            rows={1}
            onKeyPress={e =>
              e.key === 'Enter' && !e.shiftKey && handleSendMessage()
            }
          />
          <Button
            variant="default"
            size="icon"
            onClick={handleSendMessage}
            disabled={!newMessage.trim() || sending}
            className="h-10 w-10"
          >
            <Send className="h-5 w-5" />
          </Button>
        </div>
      </CardFooter>
    </div>
  )
}

// Helper function to format file size
function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

export default AgentChat
