import React, { useState } from "react";
import { QrReader } from "react-qr-reader";

interface QrScannerProps {
  onClose: () => void;
}

const QrScanner: React.FC<QrScannerProps> = ({ onClose }) => {
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleScan = (data: string | null) => {
    if (data) {
      setResult(data);
      setError(null);
    }
  };

  const handleError = (err: Error) => {
    setError(err.message || "Failed to scan QR code");
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-[1000]">
      <div className="bg-white p-4 rounded-lg w-80 max-w-full">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg font-semibold">扫描二维码</h2>
          <button onClick={onClose} className="text-gray-500 hover:text-gray-700">
            ✕
          </button>
        </div>

        <div className="mb-4">
          <QrReader
            delay={300}
            onError={handleError}
            onScan={handleScan}
            style={{ width: "100%" }}
          />
        </div>

        {error && (
          <div className="text-red-500 mb-4">
            {error}
          </div>
        )}

        {result && (
          <div className="mb-4">
            <p className="text-sm text-gray-600">扫描结果：</p>
            <p className="text-sm font-medium break-all">{result}</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default QrScanner;
