import type { IconType } from 'react-icons';
import {
  FaUserAlt,
  FaUserTie,
  FaUserGraduate,
  FaUserNinja,
  FaUserAstronaut,
  FaUserCheck,
  FaUserCircle,
  FaUserSecret,
  FaUser,
} from 'react-icons/fa';

export interface Sender {
  id: string;
  name: string;
  avatar?: IconType;
}

export const senders: Sender[] = [
  { id: 'songGe', name: '松哥', avatar: FaUserTie },
  { id: 'kelvin', name: 'Kelvin', avatar: FaUserGraduate },
  { id: 'yuanMei', name: '袁枚', avatar: FaUserCircle },
  { id: 'yingZi', name: '英子', avatar: FaUserSecret },
  { id: 'changAn', name: '常安', avatar: FaUserAlt },
  { id: 'maoZhu', name: '毛竹', avatar: FaUserNinja },
  { id: 'yingZi2', name: '影子', avatar: FaUserAstronaut },
  { id: 'luoLuo', name: '落落', avatar: FaUserCheck },
  { id: 'lebei', name: '乐呗', avatar: FaUser },
];
