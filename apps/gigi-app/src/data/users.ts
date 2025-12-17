export interface User {
  id: string;
  name: string;
  isGroup?: boolean;
  lastMessage?: string;
  lastMessageTime?: string;
  unreadCount?: number;
}

export const users = {
  me: {
    id: "lebei",
    name: "Le Bei",
    isGroup: false,
  },
  songGe: {
    id: "songGe",
    name: "Song Ge",
    lastMessage: "Are you free to play ball this weekend?",
    lastMessageTime: "09:20",
    unreadCount: 2,
  },
  kelvin: {
    id: "kelvin",
    name: "Kelvin",
    lastMessage: "OK, see you at 7 PM tomorrow night",
    lastMessageTime: "Yesterday",
  },
  yuanMei: {
    id: "yuanMei",
    name: "Yuan Mei",
    lastMessage: "I've already bought the badminton rackets",
    lastMessageTime: "14:05",
  },
  yingZi: {
    id: "yingZi",
    name: "Ying Zi",
    lastMessage: "I checked the reviews for that restaurant, they're very good",
    lastMessageTime: "08:45",
  },
  changAn: {
    id: "changAn",
    name: "Chang An",
    lastMessage: "Mom wants you to come home for dinner this weekend",
    lastMessageTime: "19:30",
  },
  maoZhu: {
    id: "maoZhu",
    name: "Mao Zhu",
    lastMessage: "Project proposal has been sent to email",
    lastMessageTime: "Saturday",
  },
  yingZi2: {
    id: "yingZi2",
    name: "Ying Zi",
    lastMessage: "See you at the usual place at 10 PM",
    lastMessageTime: "Friday",
  },
  luoLuo: {
    id: "luoLuo",
    name: "Luo Luo",
    lastMessage: "I picked up your package for you",
    lastMessageTime: "10:22",
  },
};

export const groups = {
  pingYu: {
    id: "pingYu",
    name: "Table Tennis/Badminton Booking",
    members: [
      "me",
      "yuanMei",
      "yingZi",
      "changAn",
      "maoZhu",
      "yingZi2",
      "luoLuo",
    ],
    lastMessage: "Chang An: See you at the gym tomorrow at 3 PM",
    lastMessageTime: "Yesterday",
    unreadCount: 3,
    isGroup: true,
  },
  family: {
    id: "family",
    name: "Family",
    members: ["me", "changAn", "kelvin"],
    lastMessage: "Kelvin: Mom said to bring some local specialties back",
    lastMessageTime: "12:30",
    isGroup: true,
  },
};

export const allChats: User[] = [
  groups.pingYu,
  groups.family,
  users.songGe,
  users.kelvin,
  users.yuanMei,
  users.yingZi,
  users.changAn,
  users.maoZhu,
  users.yingZi2,
  users.luoLuo,
];

export const contacts = [
  users.changAn,
  users.kelvin,
  users.luoLuo,
  users.maoZhu,
  users.songGe,
  users.yuanMei,
  users.yingZi,
  users.yingZi2,
];
