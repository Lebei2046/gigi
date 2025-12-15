import { useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
  CardContent,
  CardFooter,
} from '@/components/ui/card'
import { useAppDispatch } from '@/store'
import { loadAuthData } from '@/store/authSlice'
import { useSignupContext } from '../context/SignupContext'

export default function SignupFinish() {
  const navigate = useNavigate()
  const appDispatch = useAppDispatch()
  const {
    state: { address, peerId, name, createGroup, groupName },
    saveAccountInfo,
    saveGroupInfo,
  } = useSignupContext()

  useEffect(() => {
    const saveInfo = async () => {
      await saveAccountInfo()

      // Save group info if user chose to create a group
      if (createGroup && groupName.trim()) {
        await saveGroupInfo()
      }
    }
    saveInfo()
  }, [saveAccountInfo, createGroup, groupName, saveGroupInfo])

  const handleLogin = async () => {
    await appDispatch(loadAuthData())
    navigate('/login')
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Your account created successfully</CardTitle>
        <CardDescription>
          Below is your first Gigi wallet account
        </CardDescription>
      </CardHeader>
      <CardContent>
        <h2>Account Details</h2>
        <p>Account Name: {name}</p>
        <p>Wallet Address: {address}</p>
        <p>Peer Id: {peerId}</p>

        {createGroup && groupName.trim() && (
          <div
            style={{
              marginTop: '16px',
              padding: '12px',
              backgroundColor: '#f0f9ff',
              borderRadius: '6px',
            }}
          >
            <h3>Group Created</h3>
            <p>Group Name: {groupName.trim()}</p>
            <p style={{ fontSize: '14px', color: '#666' }}>
              Your group has been created and is ready to use!
            </p>
          </div>
        )}
      </CardContent>
      <CardFooter>
        <Button color="primary" onClick={handleLogin}>
          Go to login
        </Button>
      </CardFooter>
    </Card>
  )
}
