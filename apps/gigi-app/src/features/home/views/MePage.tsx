import React from "react";
import Avatar from "../components/Avatar";

const MePage: React.FC = () => {
  const menus = [
    { icon: "👜", name: "服务", label: "" },
    { icon: "💰", name: "收藏", label: "" },
    { icon: "🖼️", name: "相册", label: "" },
    { icon: "💳", name: "卡包", label: "" },
    { icon: "😊", name: "表情", label: "" },
    { icon: "⚙️", name: "设置", label: "" },
  ];

  return (
    <div className="flex flex-col h-full bg-gray-100">
      {/* 个人信息区域 */}
      <div className="bg-white py-6 px-4 flex items-center">
        <div className="flex-shrink-0 mr-4">
          <Avatar name="乐呗" size="lg" />
        </div>

        <div className="flex-1">
          <div className="flex justify-between items-center">
            <h2 className="text-xl font-semibold">乐呗</h2>
          </div>
          <p className="text-gray-500 mt-1">微信号: lebay999</p>
        </div>
      </div>

      {/* 菜单区域 */}
      <div className="mt-4 bg-white">
        {menus.map((menu, index) => (
          <div
            key={menu.name}
            className={`flex items-center py-4 px-4 hover:bg-gray-50 ${
              index < menus.length - 1 ? "border-b border-gray-100" : ""
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
