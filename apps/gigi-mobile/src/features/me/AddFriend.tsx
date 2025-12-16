import { useState } from 'react'
import QRCode from 'react-qr-code'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import QrScanner from '@/components/QrScanner'
import { addContact } from '@/models/contact'

interface AddFriendProps {
  name: string
  peerId: string
}

export default function AddFriend({ name, peerId }: AddFriendProps) {
  const [showQrScanner, setShowQrScanner] = useState(false)
  const qrData = encodeURI(JSON.stringify({ name, peerId }))

  const handleOnClose = (result: string | null) => {
    if (result) {
      const value = decodeURI(result)
      try {
        const obj = JSON.parse(value)
        if (obj.name && obj.peerId) {
          addContact(obj.name, obj.peerId)
        }
      } catch (error) {
        console.log(error)
      }
    }
    setShowQrScanner(false)
  }

  const handleScanClick = (e: React.MouseEvent) => {
    e.stopPropagation()
    setShowQrScanner(true)
  }

  return (
    <div className="p-6 bg-gray-50 h-full">
      <div className="text-center space-y-6">
        {/* Header */}
        <div className="space-y-2">
          <h2 className="text-xl font-bold text-gray-900">
            Share Your QR Code
          </h2>
          <p className="text-gray-600 text-sm px-4">
            Show this QR code to friends so they can add you as a contact
          </p>
        </div>

        {/* QR Code Card */}
        <div className="bg-white rounded-2xl shadow-lg border border-gray-100 p-6 space-y-4">
          <div className="flex justify-center">
            <div className="p-4 bg-white rounded-xl border-2 border-gray-200">
              <QRCode
                value={qrData}
                size={200}
                level="H"
                fgColor="#000000"
                bgColor="#ffffff"
              />
            </div>
          </div>

          <div className="text-center space-y-2">
            <div className="font-semibold text-gray-900">{name}</div>
            <div className="text-xs text-gray-500 font-mono bg-gray-100 px-3 py-2 rounded-lg inline-block max-w-[200px] truncate">
              {peerId}
            </div>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="space-y-3">
          <Button
            onClick={handleScanClick}
            className="w-full py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-xl transition-colors duration-200 flex items-center justify-center gap-2"
          >
            <svg
              className="w-5 h-5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z"
              ></path>
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M15 13a3 3 0 11-6 0 3 3 0 016 0z"
              ></path>
            </svg>
            Scan Friend's QR Code
          </Button>
        </div>
      </div>

      {showQrScanner && (
        <div className="fixed inset-0 z-50">
          <QrScanner onClose={handleOnClose} />
        </div>
      )}
    </div>
  )
}
