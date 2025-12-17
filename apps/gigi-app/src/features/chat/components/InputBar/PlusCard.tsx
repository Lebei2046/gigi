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
    { name: 'Image', icon: <FaImage /> },
    { name: 'Camera', icon: <FaCamera /> },
    { name: 'Voice Call', icon: <FaPhoneAlt /> },
    { name: 'Location', icon: <FaMapMarkerAlt /> },
    { name: 'Red Packet', icon: <FaMoneyBillWave /> },
    { name: 'Gift', icon: <FaGift /> },
    { name: 'Transfer', icon: <FaMoneyBillWave /> },
    { name: 'Voice Input', icon: <FaKeyboard /> },
    { name: 'Favorites', icon: <FaCommentAlt /> },
  ];

  return (
    <div className="bg-white rounded-lg border p-3">
      <div className="grid grid-cols-4 gap-3">
        {actions.map((action) => (
          <button
            key={action.name}
            type="button"
            // Add detailed event handling
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
