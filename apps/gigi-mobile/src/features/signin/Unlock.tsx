import { useState, useCallback, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardAction,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { loginWithP2P, setError } from '@/store/authSlice'
import { useAppDispatch, useAppSelector } from '@/store'

export default function Unlock() {
  const navigate = useNavigate()
  const dispatch = useAppDispatch()
  const { error } = useAppSelector((state: { auth: any }) => state.auth)
  const [password, setPassword] = useState('')
  const [isLoading, setIsLoading] = useState(false)

  // Clear error when user starts typing
  useEffect(() => {
    if (error && password.length > 0) {
      dispatch(setError(''))
    }
  }, [password, error, dispatch])

  // Validate password input
  const isValidPassword = password.trim().length >= 1

  // Memoize handler to prevent unnecessary re-renders
  const handleUnlock = useCallback(async () => {
    if (!isValidPassword) return

    setIsLoading(true)
    try {
      const result = await dispatch(loginWithP2P(password))
      if (result?.payload?.success) {
        navigate('/')
      } else if (result?.payload?.error) {
        dispatch(setError(result.payload.error))
      }
    } finally {
      setIsLoading(false)
    }
  }, [password, dispatch, navigate, isValidPassword])

  // Handle form submission
  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault()
      handleUnlock()
    },
    [handleUnlock]
  )

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-gray-50 px-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">
            Welcome Back
          </h1>
          <p className="text-gray-600">
            Enter your password to unlock your account
          </p>
        </div>

        <div className="bg-white rounded-2xl shadow-lg border border-gray-100 p-6 space-y-6">
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium text-gray-700">
                Password
              </label>
              <Input
                type="password"
                placeholder="Enter your password"
                value={password}
                onChange={e => setPassword(e.target.value)}
                autoComplete="current-password"
                aria-label="Password"
                disabled={isLoading}
                className="w-full py-3 px-4 border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
              {error && (
                <div className="bg-red-50 border border-red-200 rounded-lg p-3">
                  <p className="text-red-600 text-sm" role="alert">
                    <span className="font-medium">⚠️ {error}</span>
                  </p>
                </div>
              )}
            </div>

            <Button
              type="submit"
              disabled={isLoading || !isValidPassword}
              className="w-full py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed text-white font-medium rounded-lg transition-all duration-200"
              aria-busy={isLoading}
            >
              {isLoading ? (
                <span className="flex items-center justify-center">
                  <svg
                    className="animate-spin -ml-1 mr-3 h-5 w-5 text-white"
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                  >
                    <circle
                      className="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      strokeWidth="4"
                    ></circle>
                    <path
                      className="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    ></path>
                  </svg>
                  Unlocking...
                </span>
              ) : (
                'Unlock Account'
              )}
            </Button>
          </form>

          <div className="text-center pt-4 border-t border-gray-100">
            <Button
              variant="link"
              onClick={() => navigate('/reset')}
              className="text-blue-600 hover:text-blue-700 text-sm font-medium"
            >
              Forgot password?
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}
