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
  FaUser as MeIcon
} from 'react-icons/fa';
import Me from "../me/Me"

export default function Home() {
  const mePage = useMemo(() => (
    <Me />
  ), []);

  return (
    <div className="flex flex-col w-full gap-6 pb-[calc(4rem+env(safe-area-inset-bottom))]">
      <Tabs defaultValue="chat" className="w-full">
        <TabsContent value="chat" className="flex-grow w-full">
          Chat
        </TabsContent>
        <TabsContent value="archive" className="flex-grow w-full">
          Files
        </TabsContent>
        <TabsContent value="notes" className="flex-grow w-full">
          Notes
        </TabsContent>
        <TabsContent value="me" className="flex-grow w-full">
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