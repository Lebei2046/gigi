import React, { useState, useEffect, useRef } from "react";
import { HiOutlineSearch, HiPlusCircle, HiUserGroup, HiUserAdd } from "react-icons/hi";
import { FaQrcode } from "react-icons/fa";
import QrScanner from "./QrScanner";

interface TopBarProps {
  title: string;
}

const TopBar: React.FC<TopBarProps> = ({ title }) => {
  const [showMenu, setShowMenu] = useState(false);
  const [showQrScanner, setShowQrScanner] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setShowMenu(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, []);

  return (
    <div className="sticky top-0 z-10 bg-gray-100 px-4 py-3 flex items-center">
      {/* 左侧占位元素 */}
      <div className="flex-1"></div>

      {/* 居中标题 */}
      <div className="flex-1 flex justify-center">
        <span className="text-xl font-semibold">{title}</span>
      </div>

      {/* 右侧图标 */}
      <div className="flex-1 flex justify-end space-x-3">
        <HiOutlineSearch className="w-6 h-6 text-gray-600" />
        <HiPlusCircle
          className="w-6 h-6 text-gray-600 cursor-pointer"
          onClick={() => setShowMenu(!showMenu)}
        />
        {showMenu && (
          <div ref={menuRef} className="absolute right-4 top-12 bg-white shadow-lg rounded-md py-1 z-50 w-40">
            <div className="px-4 py-2 hover:bg-gray-100 cursor-pointer flex items-center">
              <HiUserGroup className="mr-2" />
              发起群聊
            </div>
            <div className="px-4 py-2 hover:bg-gray-100 cursor-pointer flex items-center">
              <HiUserAdd className="mr-2" />
              添加朋友
            </div>
            <div
              className="px-4 py-2 hover:bg-gray-100 cursor-pointer flex items-center"
              onClick={() => {
                setShowMenu(false);
                setShowQrScanner(true);
              }}
            >
              <FaQrcode className="mr-2" />
              扫一扫
            </div>
          </div>
        )}
      </div>
      {showQrScanner && (
        <QrScanner
          onClose={() => setShowQrScanner(false)}
        />
      )}
    </div>
  );
};

export default TopBar;
