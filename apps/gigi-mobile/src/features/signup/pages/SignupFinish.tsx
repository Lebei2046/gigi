import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { useAppDispatch } from '@/store'
import { loadAuthData } from '@/store/authSlice'
import { useSignupContext } from '../context/SignupContext'

export default function SignupFinish() {
  const navigate = useNavigate()
  const appDispatch = useAppDispatch()
  const [isSaving, setIsSaving] = useState(true)
  const [saveError, setSaveError] = useState<string | null>(null)
  const {
    state: { address, peerId, name, createGroup, groupName },
    saveAccountInfo,
    saveGroupInfo,
  } = useSignupContext()

  useEffect(() => {
    const saveInfo = async () => {
      try {
        await saveAccountInfo()

        // Save group info if user chose to create a group
        if (createGroup && groupName.trim()) {
          await saveGroupInfo()
        }
        setIsSaving(false)
      } catch (error) {
        console.error('Error saving account:', error)
        setSaveError(
          error instanceof Error ? error.message : 'Failed to save account'
        )
        setIsSaving(false)
      }
    }
    saveInfo()
  }, [saveAccountInfo, createGroup, groupName, saveGroupInfo])

  const handleLogin = async () => {
    await appDispatch(loadAuthData())
    navigate('/login')
  }

  if (isSaving) {
    return (
      <div className="space-y-6 flex flex-col items-center justify-center py-12">
        <div className="w-12 h-12 border-4 border-blue-600 border-t-transparent rounded-full animate-spin" />
        <p className="text-gray-600">Creating your account...</p>
      </div>
    )
  }

  if (saveError) {
    return (
      <div className="space-y-6">
        <div className="text-center space-y-2">
          <div className="mx-auto w-16 h-16 bg-red-100 rounded-full flex items-center justify-center mb-4">
            <svg
              className="w-8 h-8 text-red-600"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M6 18L18 6M6 6l12 12"
              ></path>
            </svg>
          </div>
          <h1 className="text-2xl font-bold text-gray-900">
            Failed to Create Account
          </h1>
          <p className="text-red-600">{saveError}</p>
        </div>
        <Button
          onClick={() => window.location.reload()}
          className="w-full py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg"
        >
          Try Again
        </Button>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="text-center space-y-2">
        <div className="mx-auto w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mb-4">
          <svg
            className="w-8 h-8 text-green-600"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M5 13l4 4L19 7"
            ></path>
          </svg>
        </div>
        <h1 className="text-2xl font-bold text-gray-900">
          Account Created Successfully!
        </h1>
        <p className="text-gray-600">Your new account is ready to use</p>
      </div>

      <div className="bg-white rounded-xl border border-gray-200 shadow-sm p-6 space-y-4">
        <h2 className="text-lg font-semibold text-gray-900 flex items-center gap-2">
          <span className="w-6 h-6 bg-blue-100 rounded-full flex items-center justify-center text-xs font-bold text-blue-600">
            i
          </span>
          Account Details
        </h2>

        <div className="space-y-3">
          <div className="flex justify-between items-center py-2 border-b border-gray-100">
            <span className="text-sm font-medium text-gray-600">
              Account Name
            </span>
            <span className="text-sm font-medium text-gray-900">{name}</span>
          </div>

          <div className="flex justify-between items-center py-2 border-b border-gray-100">
            <span className="text-sm font-medium text-gray-600">
              Account Address
            </span>
            <span className="text-xs font-mono text-gray-500 bg-gray-100 px-2 py-1 rounded">
              {address?.slice(0, 8)}...{address?.slice(-8)}
            </span>
          </div>

          <div className="flex justify-between items-center py-2 border-b border-gray-100">
            <span className="text-sm font-medium text-gray-600">Peer ID</span>
            <span className="text-xs font-mono text-gray-500 bg-gray-100 px-2 py-1 rounded">
              {peerId?.slice(0, 8)}...{peerId?.slice(-8)}
            </span>
          </div>
        </div>

        {createGroup && groupName.trim() && (
          <div className="mt-4 bg-green-50 border border-green-200 rounded-lg p-4">
            <h3 className="text-sm font-semibold text-green-800 flex items-center gap-2 mb-2">
              <svg
                className="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
                ></path>
              </svg>
              Group Created Successfully
            </h3>
            <p className="text-sm text-green-700 font-medium">
              {groupName.trim()}
            </p>
            <p className="text-xs text-green-600 mt-1">
              Your group has been created and is ready to use!
            </p>
          </div>
        )}
      </div>

      <Button
        onClick={handleLogin}
        className="w-full py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-all duration-200"
      >
        Continue to Login
      </Button>
    </div>
  )
}
