import { useCallback, useEffect, useRef, useState } from "react";
import { FiCamera } from "react-icons/fi";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import { getAvatarUrl, storeAvatar } from "@/utils/imageStorage";

interface ChangeAvatarProps {
  address: string;
}

export default function ChangeAvatar({ address }: ChangeAvatarProps) {
  const [avatarUrl, setAvatarUrl] = useState<string | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const loadAvatar = useCallback(async () => {
    if (address) {
      const url = await getAvatarUrl(address);
      setAvatarUrl(url);
    }
  }, [address]);

  useEffect(() => {
    loadAvatar();
  }, [loadAvatar]);


  const handleAvatarClick = () => {
    fileInputRef.current?.click();
  };

  const handleAvatarUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file && file.type.startsWith('image/') && address) {
      try {
        setIsUploading(true);
        await storeAvatar(address, file);
        await loadAvatar();
      } catch (error) {
        console.error('Failed to upload avatar:', error);
        alert('Failed to upload avatar');
      } finally {
        setIsUploading(false);
        if (fileInputRef.current) {
          fileInputRef.current.value = '';
        }
      }
    }
  };

  return (
    <div
      className="w-16 h-16 rounded-full bg-gray-300 flex items-center justify-center mr-4 cursor-pointer relative"
      onClick={handleAvatarClick}
    >
      {
        isUploading
          ? (<div className="w-6 h-6 border-t-2 border-blue-500 rounded-full animate-spin"></div>)
          : (
            <Avatar>
              <AvatarImage src={avatarUrl || "https://github.com/shadcn.png"} />
              <AvatarFallback>CN</AvatarFallback>
            </Avatar>
          )
      }
      <div className="absolute bottom-0 right-3 bg-white rounded-full p-1 shadow">
        <FiCamera className="h-4 w-4 text-gray-600" />
      </div>
      <input
        type="file"
        ref={fileInputRef}
        accept="image/*"
        onChange={handleAvatarUpload}
        style={{ display: 'none' }}
      />
    </div>
  )
}