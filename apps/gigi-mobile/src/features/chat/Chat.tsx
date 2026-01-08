import { useEffect } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import { useAppDispatch, useAppSelector } from '@/store'
import { addLog } from '@/store/logsSlice'
import { setShowShareDrawer, subscribeToGroupsAsync } from '@/store/chatSlice'
import { ChatHeader, ErrorState, LoadingState } from './chat/layout'
import {
  DirectChatsSection,
  GroupsSection,
  GroupShareNotifications,
  ShareDrawer,
} from './chat/sections'
import {
  useChatInitialization,
  useChatDataRefresh,
  useChatEventListeners,
  usePeerActions,
  useGroupActions,
} from './chat/hooks'

export default function Chat() {
  const navigate = useNavigate()
  const location = useLocation()
  const dispatch = useAppDispatch()

  // Data hooks
  const {
    peers,
    chats,
    groups,
    latestMessages,
    groupShareNotifications,
    showShareDrawer,
    selectedGroup,
    loading,
    error,
    componentError,
    loadChats,
  } = useChatInitialization()

  // Event listeners
  useChatEventListeners()

  // Refresh handling
  useChatDataRefresh()

  // Subscribe to joined groups
  useEffect(() => {
    dispatch(subscribeToGroupsAsync())
  }, [dispatch])

  // Refresh when location changes (navigating back from ChatRoom)
  useEffect(() => {
    loadChats()
  }, [location.key])

  // Action handlers
  const { handlePeerClick } = usePeerActions()
  const {
    handleShareGroup,
    handleSendShareToPeer,
    handleAcceptGroupShare,
    handleIgnoreGroupShare,
    handleClearMessages,
  } = useGroupActions()

  // Error and loading states
  if (componentError) {
    return <ErrorState error={componentError} />
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
    return <LoadingState />
  }

  return (
    <div className="flex flex-col h-full bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200 px-4 py-4">
        <h2 className="text-2xl font-bold text-gray-900">Messages</h2>
      </div>

      <div className="flex-1 overflow-y-auto px-4 py-4">
        {/* Group Share Notifications */}
        <GroupShareNotifications
          notifications={groupShareNotifications}
          onAccept={handleAcceptGroupShare}
          onIgnore={handleIgnoreGroupShare}
        />

        {/* Groups Section */}
        <GroupsSection
          groups={groups}
          chats={chats}
          latestMessages={latestMessages}
          onGroupClick={groupId => navigate(`/chat/${groupId}`)}
          onShare={handleShareGroup}
          onClearMessages={handleClearMessages}
        />

        {/* Direct Chats Section */}
        <DirectChatsSection
          peers={peers}
          chats={chats}
          latestMessages={latestMessages}
          onPeerClick={handlePeerClick}
          onClearMessages={handleClearMessages}
        />

        {/* Share Drawer */}
        <ShareDrawer
          isOpen={showShareDrawer}
          selectedGroup={selectedGroup}
          peers={peers}
          onClose={() => dispatch(setShowShareDrawer(false))}
          onShare={handleSendShareToPeer}
        />
      </div>
    </div>
  )
}
