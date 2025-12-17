import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { useAppDispatch } from '@/store'
import { resetAuth } from '@/store/authSlice'

export default function ResetAccount() {
  const navigate = useNavigate()
  const dispatch = useAppDispatch()
  const [checked, setChecked] = useState(false)

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-red-50 to-gray-50 px-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
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
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
              ></path>
            </svg>
          </div>
          <h1 className="text-3xl font-bold text-gray-900 mb-2">
            Reset Account
          </h1>
          <p className="text-gray-600">
            You are about to permanently delete your account
          </p>
        </div>

        <div className="bg-white rounded-2xl shadow-lg border border-gray-100 p-6 space-y-6">
          <Alert variant="default" className="bg-red-50 border-red-200">
            <AlertTitle className="text-red-800 font-semibold flex items-center gap-2">
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
                  d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                ></path>
              </svg>
              Warning: Destructive Action
            </AlertTitle>
            <AlertDescription className="text-red-700">
              Resetting your account will permanently delete all your data,
              including your account information and transaction history. This
              action cannot be undone. Please ensure you have backed up your
              recovery phrase before proceeding.
            </AlertDescription>
          </Alert>

          <div className="space-y-4">
            <div className="bg-gray-50 rounded-lg p-4 border border-gray-200">
              <h3 className="text-sm font-semibold text-gray-900 mb-2">
                What will be deleted:
              </h3>
              <ul className="text-sm text-gray-600 space-y-1">
                <li className="flex items-center gap-2">
                  <span className="text-red-500">✕</span> Account information
                </li>
                <li className="flex items-center gap-2">
                  <span className="text-red-500">✕</span> Transaction history
                </li>
                <li className="flex items-center gap-2">
                  <span className="text-red-500">✕</span> Chat messages
                </li>
                <li className="flex items-center gap-2">
                  <span className="text-red-500">✕</span> Group memberships
                </li>
              </ul>
            </div>

            <div className="flex items-start space-x-3 p-4 bg-amber-50 rounded-lg border border-amber-200">
              <input
                type="checkbox"
                id="accept-risk"
                checked={checked}
                onChange={() => setChecked(!checked)}
                className="mt-1 w-4 h-4 text-red-600 bg-gray-100 border-gray-300 rounded focus:ring-red-500 focus:ring-2"
              />
              <label
                htmlFor="accept-risk"
                className="text-sm font-medium text-gray-700 leading-relaxed"
              >
                I understand this action is permanent and irreversible. I have
                backed up my recovery phrase and accept all risks.
              </label>
            </div>
          </div>

          <div className="flex gap-3">
            <Button
              variant="outline"
              onClick={() => navigate(-1)}
              className="flex-1 py-3 border-gray-300 text-gray-700 hover:bg-gray-50"
            >
              Cancel
            </Button>
            <Button
              className="flex-1 py-3 bg-red-600 hover:bg-red-700 text-white font-medium rounded-lg transition-all duration-200 disabled:bg-gray-300 disabled:cursor-not-allowed"
              disabled={!checked}
              onClick={() => {
                dispatch(resetAuth())
                navigate('/signup')
              }}
            >
              Reset Account
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}
