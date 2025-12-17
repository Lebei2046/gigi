import React from "react";
import { useNavigate } from "react-router-dom";
import Avatar from "../../../components/Avatar";
import { useAppSelector } from "../../../store";
import { FaQrcode } from "react-icons/fa";

const MePage: React.FC = () => {
  const navigate = useNavigate();
  const menus = [
    { icon: "👜", name: "Services", label: "" },
    { icon: "💰", name: "Favorites", label: "" },
    { icon: "🖼️", name: "Album", label: "" },
    { icon: "💳", name: "Wallet", label: "" },
    { icon: "😊", name: "Emojis", label: "" },
    { icon: "⚙️", name: "Settings", label: "" },
  ];
  const { name, address } = useAppSelector((state) => state.auth);

  return (
    <div className="flex flex-col h-full bg-gray-100">
      {/* Personal info area */}
      <div className="bg-white py-6 px-4 flex items-center">
        <div className="flex-shrink-0 mr-4">
          <Avatar name={name || ''} address={address || ''} size={60} />
        </div>

        <div className="flex-1">
          <div className="flex justify-between items-center">
            <h2 className="text-xl font-semibold">{name}</h2>
            <FaQrcode
              className="text-gray-500 cursor-pointer hover:text-gray-700"
              onClick={() => navigate('/me')}
            />
          </div>
          <p className="text-gray-500 mt-1">Giji ID: {address}</p>
        </div>
      </div>

      {/* Menu area */}
      <div className="mt-4 bg-white">
        {menus.map((menu, index) => (
          <div
            key={menu.name}
            className={`flex items-center py-4 px-4 hover:bg-gray-50 ${index < menus.length - 1 ? "border-b border-gray-100" : ""
              }`}
          >
            <div className="w-8 text-lg">{menu.icon}</div>
            <span className="flex-1">{menu.name}</span>
            {menu.label && (
              <span className="text-sm text-gray-500">{menu.label}</span>
            )}
            <div className="w-4 text-gray-400">{"›"}</div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default MePage;
