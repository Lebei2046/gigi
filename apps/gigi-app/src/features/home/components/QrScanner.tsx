import React from "react";
import { Scanner, type IDetectedBarcode } from "@yudiel/react-qr-scanner";

interface QrScannerProps {
  onClose: (value: string | null) => void;
}

const QrScanner: React.FC<QrScannerProps> = ({ onClose }) => {

  const handleScan = (data: IDetectedBarcode[] | null) => {
    if (data) {
      onClose(data[0].rawValue);
    } else {
      onClose(null);
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-[1000]">
      <div className="bg-white p-4 rounded-lg w-80 max-w-full">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg font-semibold">扫描二维码</h2>
          <button onClick={() => onClose(null)} className="text-gray-500 hover:text-gray-700">
            ✕
          </button>
        </div>

        <div className="mb-4">
          <Scanner
            onScan={(result) => handleScan(result)}
            constraints={{ facingMode: "environment" }}
          />
        </div>
      </div>
    </div>
  );
};

export default QrScanner;
