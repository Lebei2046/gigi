import React from "react";
import { useNavigate } from "react-router-dom";
import QRCode from "react-qr-code";
import { FiArrowLeft } from "react-icons/fi";
import { useAppSelector } from "../../store";

const Me: React.FC = () => {
  const navigate = useNavigate();
  const { name, address } = useAppSelector((state) => state.auth);
  const qrData = encodeURI(JSON.stringify({ name, address }));

  return (
    <div className="flex flex-col items-center p-6 bg-white rounded-lg shadow-md">
      <div className="flex items-center w-full mb-6">
        <button className="mr-4" onClick={() => navigate(-1)}>
          <FiArrowLeft className="h-6 w-6" />
        </button>
        <h2 className="text-xl font-semibold">个人中心</h2>
      </div>
      <div className="flex items-center mb-4">
        <div className="w-10 h-10 rounded-full bg-gray-300 flex items-center justify-center mr-4">
          <span className="text-gray-600">A</span>
        </div>
        <div>
          <p className="text-lg font-medium">{name}</p>
          <p className="text-sm text-gray-600">{address}</p>
        </div>
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
