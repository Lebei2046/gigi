import React from 'react';
import {
  FaImage,
  FaCamera,
  FaPhoneAlt,
  FaMapMarkerAlt,
  FaGift,
  FaMoneyBillWave,
  FaKeyboard,
  FaCommentAlt,
} from 'react-icons/fa';

interface PlusCardProps {
  onSelect: (action: string) => void;
}

const PlusCard: React.FC<PlusCardProps> = ({ onSelect }) => {
  const actions = [
    { name: '图片', icon: <FaImage /> },
    { name: '拍摄', icon: <FaCamera /> },
    { name: '语音通话', icon: <FaPhoneAlt /> },
    { name: '位置', icon: <FaMapMarkerAlt /> },
    { name: '红包', icon: <FaMoneyBillWave /> },
    { name: '礼物', icon: <FaGift /> },
    { name: '转账', icon: <FaMoneyBillWave /> },
    { name: '语音输入', icon: <FaKeyboard /> },
    { name: '收藏', icon: <FaCommentAlt /> },
  ];

  return (
    <div className="bg-white rounded-lg border p-3">
      <div className="grid grid-cols-4 gap-3">
        {actions.map((action) => (
          <button
            key={action.name}
            type="button"
            // 添加详细事件处理
            onClick={() => {
              onSelect(action.name);
            }}
            className="flex flex-col items-center p-2 hover:bg-gray-100 rounded-lg transition-colors"
          >
            <div className="text-2xl text-blue-500 mb-1">{action.icon}</div>
            <span className="text-xs text-gray-500">{action.name}</span>
          </button>
        ))}
      </div>
    </div>
  );
};

export default PlusCard;
