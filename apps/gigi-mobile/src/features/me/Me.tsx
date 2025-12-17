import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import ContactList from './ContactList'
import ChangeAvatar from './ChangeAvatar'
import AddFriend from './AddFriend'
import { useAppSelector } from '@/store'
import { formatShortPeerId } from '@/utils/peerUtils'

export default function Me() {
  const { name, peerId } = useAppSelector(state => state.auth)

  return (
    <div className="flex flex-col h-full bg-gray-50">
      {/* Personal Information Header */}
      <div className="bg-gradient-to-br from-blue-600 to-purple-700 p-6 pb-8">
        <div className="flex items-center space-x-4">
          <div className="flex-shrink-0">
            <ChangeAvatar peerId={peerId || ''} name={name || ''} />
          </div>

          <div className="flex-1 min-w-0">
            <h1 className="text-2xl font-bold text-white truncate mb-1">
              {name || 'Anonymous User'}
            </h1>
            <div className="flex items-center gap-2">
              <span className="text-blue-100 text-sm">Peer ID:</span>
              <p className="text-white text-sm font-mono bg-white/20 px-2 py-1 rounded truncate max-w-[200px]">
                {formatShortPeerId(peerId) || 'Unknown'}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Tab Content */}
      <div className="flex-1 bg-white rounded-t-3xl -mt-4 relative">
        <div className="bg-white rounded-t-3xl">
          <div className="w-12 h-1 bg-gray-300 rounded-full mx-auto mt-3 mb-4"></div>

          <Tabs defaultValue="contacts" className="w-full">
            <TabsList className="grid grid-cols-3 w-full mx-4 mb-6 bg-gray-100 rounded-xl p-1">
              <TabsTrigger
                value="contacts"
                className="data-[state=active]:bg-white data-[state=active]:shadow-sm rounded-lg text-sm font-medium transition-all duration-200"
              >
                Contacts
              </TabsTrigger>
              <TabsTrigger
                value="2d-code"
                className="data-[state=active]:bg-white data-[state=active]:shadow-sm rounded-lg text-sm font-medium transition-all duration-200"
              >
                QR Code
              </TabsTrigger>
              <TabsTrigger
                value="settings"
                className="data-[state=active]:bg-white data-[state=active]:shadow-sm rounded-lg text-sm font-medium transition-all duration-200"
              >
                Settings
              </TabsTrigger>
            </TabsList>

            <TabsContent value="contacts" className="mt-0">
              <div className="h-[calc(100vh-250px)]">
                <ContactList />
              </div>
            </TabsContent>

            <TabsContent value="2d-code" className="mt-0">
              <div className="h-[calc(100vh-250px)]">
                <AddFriend name={name || ''} peerId={peerId || ''} />
              </div>
            </TabsContent>

            <TabsContent value="settings" className="mt-0">
              <div className="p-6 h-[calc(100vh-250px)]">
                <div className="bg-gray-50 rounded-xl p-6 text-center">
                  <div className="w-16 h-16 bg-gray-200 rounded-full flex items-center justify-center mx-auto mb-4">
                    <svg
                      className="w-8 h-8 text-gray-400"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth="2"
                        d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                      ></path>
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth="2"
                        d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                      ></path>
                    </svg>
                  </div>
                  <h3 className="text-lg font-semibold text-gray-900 mb-2">
                    Settings Coming Soon
                  </h3>
                  <p className="text-gray-600 text-sm">
                    Settings and preferences will be available in the next
                    update.
                  </p>
                </div>
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </div>
    </div>
  )
}
