import React, { useState, useEffect, useRef } from "react";
import { HiOutlineSearch, HiPlusCircle, HiUserGroup, HiUserAdd } from "react-icons/hi";
import { FaQrcode } from "react-icons/fa";
import QrScanner from "./QrScanner";
import { addContact } from "../../../models/contact";

interface TopBarProps {
  title: string;
  menuOpen: boolean;
  setMenuOpen: (open: boolean) => void;
}

const TopBar: React.FC<TopBarProps> = ({ title, menuOpen, setMenuOpen }) => {
  const [showQrScanner, setShowQrScanner] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setMenuOpen(false);
      }
    };

    if (menuOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [menuOpen, setMenuOpen]);

  const handleOnClose = (result: string | null) => {
    if (result) {
      const value = decodeURI(result);
      try {
        const obj = JSON.parse(value);
        if (obj.name && obj.address) {
          addContact(obj.name, obj.address);
        }
      } catch (error) {
        console.log(error);
      }
    }
    setShowQrScanner(false);
  }

  const handleMenuTriggerClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setMenuOpen(!menuOpen);
  };

  const handleScanClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setMenuOpen(false);
    setShowQrScanner(true);
  };

  return (
    <div className="sticky top-0 z-50 bg-gray-100 px-4 py-3 flex items-center">
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
          onClick={handleMenuTriggerClick}
        />
        {menuOpen && (
          <div
            ref={menuRef}
            className="absolute right-4 top-12 bg-white shadow-lg rounded-md py-1 z-50 w-40"
            onClick={(e) => e.stopPropagation()}
          >
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
              onClick={handleScanClick}
            >
              <FaQrcode className="mr-2" />
              扫一扫
            </div>
          </div>
        )}
      </div>
      {showQrScanner && (
        <QrScanner
          onClose={handleOnClose} />
      )}
    </div>
  );
};

export default TopBar;
