import { useState } from "react";
import QRCode from "react-qr-code";
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import QrScanner from "@/components/QrScanner";
import { addContact } from "@/models/contact";

interface AddFriendProps {
  name: string;
  peerId: string;
}

export default function AddFriend({ name, peerId }: AddFriendProps) {
  const [showQrScanner, setShowQrScanner] = useState(false);
  const qrData = encodeURI(JSON.stringify({ name, peerId }));

  const handleOnClose = (result: string | null) => {
    if (result) {
      const value = decodeURI(result);
      try {
        const obj = JSON.parse(value);
        if (obj.name && obj.peerId) {
          addContact(obj.name, obj.peerId);
        }
      } catch (error) {
        console.log(error);
      }
    }
    setShowQrScanner(false);
  }

  const handleScanClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowQrScanner(true);
  };

  return (
    <div className="items-center p-2 bg-gray-50 rounded">
      <Card className="w-full max-w-sm">
        <CardHeader>
          <CardTitle>Add a friend by scanning 2d-code</CardTitle>
          <CardDescription>
            Show your 2d-code to your friends, and they can add you as a friend.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <QRCode
            value={qrData}
            size={128}
            level="H"
            fgColor="#000000"
            bgColor="#ffffff"
          />
        </CardContent>
        <CardFooter className="flex-col gap-2">
          <Button variant="outline" className="w-full" onClick={handleScanClick}>
            Scan Code
          </Button>
        </CardFooter>
      </Card>
      {showQrScanner && (
        <QrScanner
          onClose={handleOnClose} />
      )}
    </div>
  );
}