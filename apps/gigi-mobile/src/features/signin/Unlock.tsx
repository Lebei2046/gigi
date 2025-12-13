import { useState } from 'react'
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

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <div className="w-full max-w-md p-4">
        <Card>
          <CardHeader>
            <CardTitle>Unlock Account</CardTitle>
            <CardDescription>Enter your password to continue</CardDescription>
            <CardAction>
              <Button variant="link" onClick={() => navigate('/reset')}>
                Forgot password?
              </Button>
            </CardAction>
          </CardHeader>
          <CardFooter className="flex flex-col">
            <div className="mb-2">
              <Input
                type="password"
                placeholder="Enter your password"
                value={password}
                onChange={e => setPassword(e.target.value)}
              />
              {error && <p>{error}</p>}
            </div>
            <div>
              <Button
                color="primary"
                disabled={isLoading}
                onClick={async () => {
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
                }}
              >
                {isLoading ? 'Initializing...' : 'Unlock'}
              </Button>
            </div>
          </CardFooter>
        </Card>
      </div>
    </div>
  )
}
