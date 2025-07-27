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
  { id: 'user1', name: '松哥', avatar: FaUserTie },
  { id: 'user2', name: 'Kelvin', avatar: FaUserGraduate },
  { id: 'user3', name: '袁枚', avatar: FaUserCircle },
  { id: 'user4', name: '英子', avatar: FaUserSecret },
  { id: 'user5', name: '常安', avatar: FaUserAlt },
  { id: 'user6', name: '毛竹', avatar: FaUserNinja },
  { id: 'user7', name: '影子', avatar: FaUserAstronaut },
  { id: 'user8', name: '落落', avatar: FaUserCheck },
  { id: 'lebei', name: '乐呗', avatar: FaUser },
];
