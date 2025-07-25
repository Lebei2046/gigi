import React, { useState, useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import QRCode from "react-qr-code";
import { FiArrowLeft, FiCamera } from "react-icons/fi";
import { useAppSelector } from "../../store";
import { getAvatarUrl, storeAvatar } from "../../utils/imageStorage";

const Me: React.FC = () => {
  const navigate = useNavigate();
  const { name, address } = useAppSelector((state) => state.auth);
  const [avatarUrl, setAvatarUrl] = useState<string | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const qrData = encodeURI(JSON.stringify({ name, address }));

  useEffect(() => {
    loadAvatar();
  }, [address]);

  const loadAvatar = async () => {
    if (address) {
      const url = await getAvatarUrl(address);
      setAvatarUrl(url);
    }
  };

  const handleBack = () => {
    // 使用 replace 而不是 push 来避免在浏览器历史记录中留下额外条目
    navigate("/", { replace: true, state: { tab: 'me' } });
  };

  const handleAvatarClick = () => {
    fileInputRef.current?.click();
  };

  const handleAvatarUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file && file.type.startsWith('image/') && address) {
      try {
        setIsUploading(true);
        await storeAvatar(address, file);
        await loadAvatar(); // 重新加载头像
      } catch (error) {
        console.error('Failed to upload avatar:', error);
        alert('头像上传失败');
      } finally {
        setIsUploading(false);
        // 重置文件输入
        if (fileInputRef.current) {
          fileInputRef.current.value = '';
        }
      }
    }
  };

  return (
    <div className="flex flex-col items-center p-6 bg-white rounded-lg shadow-md">
      <div className="flex items-center w-full mb-6">
        <button className="mr-4" onClick={handleBack}>
          <FiArrowLeft className="h-6 w-6" />
        </button>
        <h2 className="text-xl font-semibold">个人中心</h2>
      </div>
      <div className="flex items-center mb-4 relative">
        <div
          className="w-16 h-16 rounded-full bg-gray-300 flex items-center justify-center mr-4 cursor-pointer relative"
          onClick={handleAvatarClick}
        >
          {isUploading ? (
            <div className="w-6 h-6 border-t-2 border-blue-500 rounded-full animate-spin"></div>
          ) : avatarUrl ? (
            <img
              src={avatarUrl}
              alt="Avatar"
              className="w-full h-full rounded-full object-cover"
            />
          ) : (
            <span className="text-gray-600">A</span>
          )}
          <div className="absolute bottom-0 right-3 bg-white rounded-full p-1 shadow">
            <FiCamera className="h-4 w-4 text-gray-600" />
          </div>
        </div>
        <div>
          <p className="text-lg font-medium">{name}</p>
          <p className="text-sm text-gray-600">{address}</p>
        </div>
        <input
          type="file"
          ref={fileInputRef}
          accept="image/*"
          onChange={handleAvatarUpload}
          style={{ display: 'none' }}
        />
      </div>
      <div className="p-2 bg-gray-50 rounded">
        <QRCode
          value={qrData}
          size={128}
          level="H"
          fgColor="#000000"
          bgColor="#ffffff"
        />
      </div>
    </div>
  );
};

export default Me;
