import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import ChangeAvatar from "./ChangeAvatar";
import AddFriend from "./AddFriend";
import { useAppSelector } from "@/store";

export default function Me() {
  const { name, address } = useAppSelector((state) => state.auth);

  return (
    <div className="flex flex-col bg-gray-100">
      {/* Personal Information */}
      <div className="bg-white py-6 px-4 flex items-center">
        <div className="flex-shrink-0 mr-0">
          <ChangeAvatar address={address || ''} />
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex justify-between items-center">
            <h2 className="text-xl font-semibold truncate">{name}</h2>
          </div>
          <p className="text-gray-500 mt-1 truncate">Gigi: {address}</p>
        </div>
      </div>

      {/* Menus */}
      <div className="mt-4 bg-white">
        <Tabs defaultValue="contacts" className="w-full">
          <TabsList className="flex w-full">
            <TabsTrigger value="contacts">
              Contacts
            </TabsTrigger>
            <TabsTrigger value="2d-code">
              2D Code
            </TabsTrigger>
            <TabsTrigger value="settings">
              Settings
            </TabsTrigger>
          </TabsList>
          <TabsContent value="contacts" className="flex-grow w-full">
            Contacts
          </TabsContent>
          <TabsContent value="2d-code" className="flex-grow w-full">
            <AddFriend name={name || ''} address={address || ''} />
          </TabsContent>
          <TabsContent value="settings" className="flex-grow w-full">
            Settings
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}