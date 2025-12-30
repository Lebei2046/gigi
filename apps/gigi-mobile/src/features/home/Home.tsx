import { useMemo, useEffect, useRef } from 'react'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  FaComment as ChatIcon,
  FaArchive as ArchiveIcon,
  FaStickyNote as NotesIcon,
  FaUser as MeIcon,
  FaListAlt as LogsIcon,
} from 'react-icons/fa'
import Me from '../me/Me'
import Chat from '../chat/Chat'
import P2PLogs from '@/components/P2PLogs'
import { useAppSelector, useAppDispatch } from '@/store'
import { addLog, clearLogs } from '@/store/logsSlice'
import { MessagingEvents } from '@/utils/messaging'

export default function Home() {
  const logs = useAppSelector(state => state.logs.logs)
  const dispatch = useAppDispatch()

  const mePage = useMemo(() => <Me />, [])

  const chatPage = useMemo(() => <Chat />, [])

  const logsPage = useMemo(
    () => (
      <P2PLogs
        logs={logs}
        addLog={(event, data, type) => dispatch(addLog({ event, data, type }))}
        clearLogs={() => dispatch(clearLogs())}
      />
    ),
    [logs, dispatch]
  )

  return (
    <div className="flex flex-col w-full h-full bg-gray-50 pb-[calc(4rem+env(safe-area-inset-bottom))]">
      <Tabs defaultValue="chat" className="w-full h-full flex flex-col">
        <TabsContent
          value="chat"
          className="flex-grow w-full data-[state=active]:flex data-[state=active]:flex-col m-0"
        >
          {chatPage}
        </TabsContent>
        <TabsContent
          value="logs"
          className="flex-grow w-full data-[state=active]:flex data-[state=active]:flex-col m-0"
        >
          {logsPage}
        </TabsContent>
        <TabsContent
          value="archive"
          className="flex-grow w-full data-[state=active]:flex data-[state=active]:flex-col m-0"
        >
          <div className="flex items-center justify-center h-full">
            <div className="text-center space-y-4">
              <div className="w-24 h-24 bg-gray-200 rounded-full flex items-center justify-center mx-auto">
                <ArchiveIcon className="w-12 h-12 text-gray-400" />
              </div>
              <h3 className="text-xl font-semibold text-gray-900">Files</h3>
              <p className="text-gray-600 text-sm px-6">
                File sharing and management features coming soon
              </p>
            </div>
          </div>
        </TabsContent>
        <TabsContent
          value="notes"
          className="flex-grow w-full data-[state=active]:flex data-[state=active]:flex-col m-0"
        >
          <div className="flex items-center justify-center h-full">
            <div className="text-center space-y-4">
              <div className="w-24 h-24 bg-gray-200 rounded-full flex items-center justify-center mx-auto">
                <NotesIcon className="w-12 h-12 text-gray-400" />
              </div>
              <h3 className="text-xl font-semibold text-gray-900">Notes</h3>
              <p className="text-gray-600 text-sm px-6">
                Personal notes and reminders coming soon
              </p>
            </div>
          </div>
        </TabsContent>
        <TabsContent
          value="me"
          className="flex-grow w-full data-[state=active]:flex data-[state=active]:flex-col m-0"
        >
          {mePage}
        </TabsContent>

        {/* Enhanced Bottom Navigation */}
        <div className="fixed inset-x-0 bottom-0 h-[calc(4rem+env(safe-area-inset-bottom))] bg-white border-t border-gray-200 shadow-lg">
          <div className="h-full flex items-end pb-[env(safe-area-inset-bottom)]">
            <TabsList className="flex w-full h-full bg-transparent border-none">
              <TabsTrigger
                value="chat"
                className="flex-1 data-[state=active]:text-blue-600 hover:text-blue-500 transition-colors duration-200"
              >
                <div className="flex flex-col items-center py-2">
                  <ChatIcon className="w-5 h-5 mb-1" />
                  <span className="text-xs font-medium">Chat</span>
                </div>
              </TabsTrigger>
              <TabsTrigger
                value="logs"
                className="flex-1 data-[state=active]:text-blue-600 hover:text-blue-500 transition-colors duration-200"
              >
                <div className="flex flex-col items-center py-2">
                  <LogsIcon className="w-5 h-5 mb-1" />
                  <span className="text-xs font-medium">Logs</span>
                </div>
              </TabsTrigger>
              <TabsTrigger
                value="archive"
                className="flex-1 data-[state=active]:text-blue-600 hover:text-blue-500 transition-colors duration-200"
              >
                <div className="flex flex-col items-center py-2">
                  <ArchiveIcon className="w-5 h-5 mb-1" />
                  <span className="text-xs font-medium">Files</span>
                </div>
              </TabsTrigger>
              <TabsTrigger
                value="notes"
                className="flex-1 data-[state=active]:text-blue-600 hover:text-blue-500 transition-colors duration-200"
              >
                <div className="flex flex-col items-center py-2">
                  <NotesIcon className="w-5 h-5 mb-1" />
                  <span className="text-xs font-medium">Notes</span>
                </div>
              </TabsTrigger>
              <TabsTrigger
                value="me"
                className="flex-1 data-[state=active]:text-blue-600 hover:text-blue-500 transition-colors duration-200"
              >
                <div className="flex flex-col items-center py-2">
                  <MeIcon className="w-5 h-5 mb-1" />
                  <span className="text-xs font-medium">Me</span>
                </div>
              </TabsTrigger>
            </TabsList>
          </div>
        </div>
      </Tabs>
    </div>
  )
}
