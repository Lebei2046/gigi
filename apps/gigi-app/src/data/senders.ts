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
  { id: 'songGe', name: 'Song Ge', avatar: FaUserTie },
  { id: 'kelvin', name: 'Kelvin', avatar: FaUserGraduate },
  { id: 'yuanMei', name: 'Yuan Mei', avatar: FaUserCircle },
  { id: 'yingZi', name: 'Ying Zi', avatar: FaUserSecret },
  { id: 'changAn', name: 'Chang An', avatar: FaUserAlt },
  { id: 'maoZhu', name: 'Mao Zhu', avatar: FaUserNinja },
  { id: 'yingZi2', name: 'Ying Zi', avatar: FaUserAstronaut },
  { id: 'luoLuo', name: 'Luo Luo', avatar: FaUserCheck },
  { id: 'lebei', name: 'Le Bei', avatar: FaUser },
];
