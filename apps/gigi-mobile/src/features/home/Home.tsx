import { useMemo } from "react"
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs"
import {
  FaComment as ChatIcon,
  FaArchive as ArchiveIcon,
  FaStickyNote as NotesIcon,
  FaUser as MeIcon,
  FaListAlt as LogsIcon
} from 'react-icons/fa';
import Me from "../me/Me";
import P2PLogs from "@/components/P2PLogs";
import { useAppSelector, useAppDispatch } from "@/store";
import { addLog, clearLogs } from "@/store/logsSlice";

export default function Home() {
  const logs = useAppSelector((state) => state.logs.logs);
  const dispatch = useAppDispatch();

  const mePage = useMemo(() => (
    <Me />
  ), []);

  const chatPage = useMemo(() => (
    <div className="flex items-center justify-center h-full">
      <p className="text-gray-500">Chat feature coming soon</p>
    </div>
  ), []);

  const logsPage = useMemo(() => (
    <P2PLogs 
      logs={logs} 
      addLog={(event, data, type) => dispatch(addLog({ event, data, type }))}
      clearLogs={() => dispatch(clearLogs())}
    />
  ), [logs, dispatch]);

  return (
    <div className="flex flex-col w-full h-full pb-[calc(4rem+env(safe-area-inset-bottom))]">
      <Tabs defaultValue="chat" className="w-full h-full flex flex-col">
        <TabsContent value="chat" className="flex-grow w-full data-[state=active]:flex data-[state=active]:flex-col">
          {chatPage}
        </TabsContent>
        <TabsContent value="logs" className="flex-grow w-full data-[state=active]:flex data-[state=active]:flex-col">
          {logsPage}
        </TabsContent>
        <TabsContent value="archive" className="flex-grow w-full data-[state=active]:flex data-[state=active]:flex-col">
          Files
        </TabsContent>
        <TabsContent value="notes" className="flex-grow w-full data-[state=active]:flex data-[state=active]:flex-col">
          Notes
        </TabsContent>
        <TabsContent value="me" className="flex-grow w-full data-[state=active]:flex data-[state=active]:flex-col">
          {mePage}
        </TabsContent>
        <div className="fixed inset-x-0 bottom-0 h-[calc(4rem+env(safe-area-inset-bottom))] bg-background">
          <div className="h-full flex items-end pb-[env(safe-area-inset-bottom)]">
            <TabsList className="flex w-full">
              <TabsTrigger value="chat" className="flex-1">
                <div className="flex flex-col items-center text-sm">
                  <ChatIcon />
                  <span>Chat</span>
                </div>
              </TabsTrigger>
              <TabsTrigger value="logs" className="flex-1">
                <div className="flex flex-col items-center text-sm">
                  <LogsIcon />
                  <span>Logs</span>
                </div>
              </TabsTrigger>
              <TabsTrigger value="archive" className="flex-1">
                <div className="flex flex-col items-center text-sm">
                  <ArchiveIcon />
                  <span>Files</span>
                </div>
              </TabsTrigger>
              <TabsTrigger value="notes" className="flex-1">
                <div className="flex flex-col items-center text-sm">
                  <NotesIcon />
                  <span>Notes</span>
                </div>
              </TabsTrigger>
              <TabsTrigger value="me" className="flex-1">
                <div className="flex flex-col items-center text-sm">
                  <MeIcon />
                  <span>Me</span>
                </div>
              </TabsTrigger>
            </TabsList>
          </div>
        </div>
      </Tabs>
    </div>
  )
}