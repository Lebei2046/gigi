import React, { useState, useEffect, useCallback } from 'react';
import { getAvatarUrl } from '../../../utils/imageStorage';

interface AvatarProps {
  name?: string;
  size?: number;
  address?: string;
}

const Avatar: React.FC<AvatarProps> = ({
  name = 'U',
  size = 40,
  address
}) => {
  const [avatarUrl, setAvatarUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const loadAvatar = useCallback(async () => {
    if (address) {
      try {
        const url = await getAvatarUrl(address);
        setAvatarUrl(url);
      } catch (error) {
        console.error('Failed to load avatar:', error);
      } finally {
        setLoading(false);
      }
    } else {
      setLoading(false);
    }
  }, [address]);

  // 从用户名获取首字母
  const getInitials = (name: string) => {
    if (!name || typeof name !== 'string') return 'U';
    return name.charAt(0).toUpperCase();
  };

  useEffect(() => {
    loadAvatar();
  }, [loadAvatar]);


  if (loading) {
    return (
      <div
        className="rounded-full bg-gray-300 flex items-center justify-center"
        style={{ width: size, height: size }}
      >
        <div className="w-1/2 h-1/2 bg-gray-400 rounded-full"></div>
      </div>
    );
  }

  if (avatarUrl) {
    return (
      <img
        src={avatarUrl}
        alt={name}
        className="rounded-full object-cover"
        style={{ width: size, height: size }}
      />
    );
  }

  // 如果没有上传头像，则显示默认头像
  return (
    <div
      className="rounded-full bg-gray-300 flex items-center justify-center text-gray-700 font-medium"
      style={{ width: size, height: size }}
    >
      {getInitials(name)}
    </div>
  );
};

export default Avatar;
