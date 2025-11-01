import React, { useCallback, useEffect, useState } from 'react';
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import { getAvatarUrl } from "@/utils/imageStorage";

interface ContactListItemProps {
  name: string;
  address: string
  onClick: () => void;
}

const ContactListItem: React.FC<ContactListItemProps> = ({ name, address, onClick }) => {
  const [avatarUrl, setAvatarUrl] = useState<string | null>(null);

  const loadAvatar = useCallback(async () => {
    if (address) {
      const url = await getAvatarUrl(address);
      setAvatarUrl(url);
    }
  }, [address]);

  useEffect(() => {
    loadAvatar();
  }, [loadAvatar]);

  return (
    <div
      className="flex items-center p-3 border-b border-gray-200 hover:bg-gray-50 cursor-pointer"
      onClick={onClick}
    >
      <Avatar>
        <AvatarImage src={avatarUrl || "https://github.com/shadcn.png"} />
        <AvatarFallback>CN</AvatarFallback>
      </Avatar>
      <div className="flex-1 min-w-0">
        <div className="flex justify-between items-center">
          <h2 className="text-xl font-semibold truncate">{name}</h2>
        </div>
        <p className="text-gray-500 mt-1 truncate">{address}</p>
      </div>
    </div>
  );
};

export default ContactListItem;
