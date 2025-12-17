import React, { useState, useEffect, useCallback } from 'react';
import { getAvatarUrl } from '../utils/imageStorage';
import type { IconType } from 'react-icons';
import { senders } from '../data/senders';

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
  const [senderInfo, setSenderInfo] = useState<{ name: string; avatar?: IconType } | null>(null);

  // Look up sender information if address is provided
  useEffect(() => {
    if (address) {
      const sender = senders.find(s => s.id === address);
      if (sender) {
        setSenderInfo(sender);
      }
    }
  }, [address]);

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

  // Get initials from username
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

  // If there's a sender with an avatar, use it
  if (senderInfo?.avatar) {
    const SenderIcon = senderInfo.avatar;
    return (
      <div
        className="rounded-full bg-gray-300 flex items-center justify-center text-gray-700"
        style={{ width: size, height: size }}
      >
        <SenderIcon size={size * 0.6} />
      </div>
    );
  }

  // If no avatar is uploaded, show default avatar
  return (
    <div
      className="rounded-full bg-gray-300 flex items-center justify-center text-gray-700 font-medium"
      style={{ width: size, height: size }}
    >
      {getInitials(name || senderInfo?.name || 'U')}
    </div>
  );
};

export default Avatar;
