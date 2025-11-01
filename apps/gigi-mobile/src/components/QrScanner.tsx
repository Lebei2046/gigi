import React, { useState, useEffect } from "react";
import { Scanner, type IDetectedBarcode } from "@yudiel/react-qr-scanner";

interface QrScannerProps {
  onClose: (value: string | null) => void;
}

const QrScanner: React.FC<QrScannerProps> = ({ onClose }) => {
  const [error, setError] = useState<string | null>(null);
  const [permissionStatus, setPermissionStatus] = useState<PermissionState | 'unknown'>('unknown');

  const handleScan = (data: IDetectedBarcode[]) => {
    if (data && data.length > 0) {
      onClose(data[0].rawValue);
    }
  };

  const handleError = (err: unknown) => {
    console.error("QR Scanner error:", err);
    if (err instanceof Error) {
      if (err.name === 'NotAllowedError') {
        setError("Camera access denied. Please allow camera permissions in your browser settings.");
        setPermissionStatus('denied');
      } else if (err.name === 'NotFoundError') {
        setError("No camera found on this device.");
      } else {
        setError(err.message || "Failed to access camera");
      }
    } else {
      setError("An unknown error occurred");
    }
  };

  const requestCameraPermission = async () => {
    try {
      if ('permissions' in navigator) {
        const permission = await navigator.permissions.query({ name: 'camera' as PermissionName });
        setPermissionStatus(permission.state);

        permission.onchange = () => {
          setPermissionStatus(permission.state);
          if (permission.state === 'granted') {
            setError(null);
          }
        };
      }
    } catch (err) {
      console.log("Permission API not supported");
    }
  };

  const retryScanner = () => {
    setError(null);
    setPermissionStatus('unknown');
  };

  useEffect(() => {
    requestCameraPermission();
  }, []);

  return (
    <div className="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-[1000]">
      <div className="bg-white p-4 rounded-lg w-80 max-w-full">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg font-semibold">Scan QR Code</h2>
          <button onClick={() => onClose(null)} className="text-gray-500 hover:text-gray-700">
            ✕
          </button>
        </div>

        <div className="mb-4">
          {error ? (
            <div className="text-center text-red-500 p-4">
              <p className="mb-2">Camera Error: {error}</p>
              {permissionStatus === 'denied' ? (
                <p className="text-sm">
                  Please enable camera permissions in your browser settings and click Retry.
                </p>
              ) : (
                <p className="text-sm">
                  Please ensure you've granted camera permissions and you're using HTTPS.
                </p>
              )}
            </div>
          ) : (
            <div className="w-full h-64">
              <Scanner
                onScan={handleScan}
                onError={handleError}
                constraints={{
                  facingMode: "environment",
                  width: { ideal: 1280 },
                  height: { ideal: 720 }
                }}
              />
            </div>
          )}
        </div>

        {error && (
          <div className="text-center">
            <button
              onClick={retryScanner}
              className="px-4 py-2 bg-primary text-primary-foreground rounded-md"
            >
              Retry
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

export default QrScanner;