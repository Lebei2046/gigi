import type { IconType } from 'react-icons';

export interface User {
  id: string;
  name: string;
  avatar: IconType;
}

export interface Message {
  id: string;
  senderId: string;
  content: string;
  timestamp: Date;
  status?: 'sent' | 'received';
}

export interface Emoji {
  id: string;
  name: string;
  symbol: string;
  category: string;
}

export interface Contact extends User {}
