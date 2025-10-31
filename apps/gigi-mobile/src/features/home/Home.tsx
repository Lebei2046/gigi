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
    <div className="flex w-full h-screen flex-col gap-6">
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
        <TabsList className="flex w-full fixed bottom-0">
          <TabsTrigger value="chat">
            <div className="flex flex-col items-center text-sm">
              <ChatIcon />
              <span >Chat</span>
            </div>
          </TabsTrigger>
          <TabsTrigger value="archive">
            <div className="flex flex-col items-center text-sm">
              <ArchiveIcon />
              <span >Files</span>
            </div>
          </TabsTrigger>
          <TabsTrigger value="notes">
            <div className="flex flex-col items-center text-sm">
              <NotesIcon />
              <span >Notes</span>
            </div>
          </TabsTrigger>
          <TabsTrigger value="me">
            <div className="flex flex-col items-center text-sm">
              <MeIcon />
              <span >Me</span>
            </div>
          </TabsTrigger>
        </TabsList>
      </Tabs>
    </div>
  )
}
