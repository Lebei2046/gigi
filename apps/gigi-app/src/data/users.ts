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
    name: "乐呗",
    isGroup: false,
  },
  songGe: {
    id: "songGe",
    name: "松哥",
    lastMessage: "周末有空一起打球吗？",
    lastMessageTime: "09:20",
    unreadCount: 2,
  },
  kelvin: {
    id: "kelvin",
    name: "Kelvin",
    lastMessage: "好的，明晚7点见",
    lastMessageTime: "昨天",
  },
  yuanMei: {
    id: "yuanMei",
    name: "袁枚",
    lastMessage: "羽毛球拍我已经买好了",
    lastMessageTime: "14:05",
  },
  yingZi: {
    id: "yingZi",
    name: "英子",
    lastMessage: "我看了那家餐厅的评价很不错",
    lastMessageTime: "08:45",
  },
  changAn: {
    id: "changAn",
    name: "常安",
    lastMessage: "妈让你周末回家吃饭",
    lastMessageTime: "19:30",
  },
  maoZhu: {
    id: "maoZhu",
    name: "毛竹",
    lastMessage: "项目方案已发邮箱",
    lastMessageTime: "周六",
  },
  yingZi2: {
    id: "yingZi2",
    name: "影子",
    lastMessage: "晚上10点在老地方见",
    lastMessageTime: "周五",
  },
  luoLuo: {
    id: "luoLuo",
    name: "落落",
    lastMessage: "你的快递我帮你取了",
    lastMessageTime: "10:22",
  },
};

export const groups = {
  pingYu: {
    id: "pingYu",
    name: "乒羽网约球",
    members: [
      "me",
      "yuanMei",
      "yingZi",
      "changAn",
      "maoZhu",
      "yingZi2",
      "luoLuo",
    ],
    lastMessage: "常安：明天下午3点体育馆见",
    lastMessageTime: "昨天",
    unreadCount: 3,
    isGroup: true,
  },
  family: {
    id: "family",
    name: "一家子",
    members: ["me", "changAn", "kelvin"],
    lastMessage: "Kelvin：妈说让你带点特产回去",
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
