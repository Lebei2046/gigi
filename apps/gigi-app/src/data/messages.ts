export interface Message {
  id: string;
  senderId: string;
  content: string;
  timestamp: Date;
  status?: 'sent' | 'received';
}

export const initialMessages: Message[] = [
  {
    id: '1',
    senderId: 'yuanMei',
    content: '大家明天下午去打球吗？',
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 2),
  },
  {
    id: '2',
    senderId: 'lebei',
    content: '好啊，我下午3点有空',
    timestamp: new Date(Date.now() - 1000 * 60 * 60),
  },
  {
    id: '3',
    senderId: 'yingZi',
    content: '算我一个！',
    timestamp: new Date(Date.now() - 1000 * 60 * 50),
  },
  {
    id: '4',
    senderId: 'changAn',
    content: '我可能晚点到，大概4点左右',
    timestamp: new Date(Date.now() - 1000 * 60 * 30),
  },
  {
    id: '5',
    senderId: 'lebei',
    content: '没问题，我会带多几副球拍',
    timestamp: new Date(Date.now() - 1000 * 60 * 15),
  },
  {
    id: '6',
    senderId: 'maoZhu',
    content: '我来不成了，周末有家事要处理 😭',
    timestamp: new Date(Date.now() - 1000 * 60 * 10),
  },
];
