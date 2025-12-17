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
    content: 'Is everyone going to play tomorrow afternoon?',
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 2),
  },
  {
    id: '2',
    senderId: 'lebei',
    content: 'Great, I\'m free at 3 PM',
    timestamp: new Date(Date.now() - 1000 * 60 * 60),
  },
  {
    id: '3',
    senderId: 'yingZi',
    content: 'Count me in!',
    timestamp: new Date(Date.now() - 1000 * 60 * 50),
  },
  {
    id: '4',
    senderId: 'changAn',
    content: 'I might be late, around 4 PM',
    timestamp: new Date(Date.now() - 1000 * 60 * 30),
  },
  {
    id: '5',
    senderId: 'lebei',
    content: 'No problem, I\'ll bring extra rackets',
    timestamp: new Date(Date.now() - 1000 * 60 * 15),
  },
  {
    id: '6',
    senderId: 'maoZhu',
    content: 'I can\'t make it, have family matters to handle 😭',
    timestamp: new Date(Date.now() - 1000 * 60 * 10),
  },
];
