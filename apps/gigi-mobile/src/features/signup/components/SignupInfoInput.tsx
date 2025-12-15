import { useState, useEffect, type ChangeEvent } from 'react'
import { Input } from '@/components/ui/input'
import { useSignupContext } from '../context/SignupContext'

export default function SignupInfoInput() {
  const STEP: number = 3

  const {
    state: { name, password, createGroup, groupName },
    dispatch,
  } = useSignupContext()
  const [confirmPassword, setConfirmPassword] = useState('')
  const [passwordStrength, setPasswordStrength] = useState(0)
  const [showWarning, setShowWarning] = useState(false)

  const handleNameChange = (e: ChangeEvent<HTMLInputElement>) => {
    dispatch({ type: 'SET_NAME', payload: e.target.value })
  }

  const handlePasswordChange = (e: ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value
    dispatch({ type: 'SET_PASSWORD', payload: value })
    const strength = Math.min(value.length * 10, 100)
    setPasswordStrength(strength)
  }

  const handleConfirmPasswordChange = (e: ChangeEvent<HTMLInputElement>) => {
    setConfirmPassword(e.target.value)
  }

  const handleCreateGroupChange = (e: ChangeEvent<HTMLInputElement>) => {
    dispatch({ type: 'SET_CREATE_GROUP', payload: e.target.checked })
    if (!e.target.checked) {
      dispatch({ type: 'SET_GROUP_NAME', payload: '' })
    }
  }

  const handleGroupNameChange = (e: ChangeEvent<HTMLInputElement>) => {
    dispatch({ type: 'SET_GROUP_NAME', payload: e.target.value })
  }

  useEffect(() => {
    const isMatch = confirmPassword === password
    const isWarning = confirmPassword !== '' && !isMatch
    const nextEnabled =
      password !== '' &&
      confirmPassword !== '' &&
      name !== '' &&
      isMatch &&
      (!createGroup || (createGroup && groupName.trim() !== ''))
    setShowWarning(isWarning)
    dispatch({
      type: 'SET_STEP_CHECKED',
      payload: { index: STEP, checked: nextEnabled },
    })
  }, [password, confirmPassword, name, createGroup, groupName, dispatch])

  return (
    <div>
      <h1>Create password for encryption</h1>
      <p>This password will be used to unlock your wallet and account.</p>

      <div>
        <label>Account Name</label>
        <Input
          type="text"
          placeholder="Enter your account name"
          value={name}
          onChange={handleNameChange}
        />
      </div>

      <div>
        <label>Password</label>
        <Input
          type="password"
          placeholder="Enter your password"
          value={password}
          onChange={handlePasswordChange}
        />
        {/* Replaced Progress with a simple div-based progress bar */}
        <div
          style={{
            width: '100%',
            backgroundColor: '#e0e0e0',
            borderRadius: '4px',
          }}
        >
          <div
            style={{
              width: `${passwordStrength}%`,
              height: '8px',
              backgroundColor:
                passwordStrength > 70
                  ? 'green'
                  : passwordStrength > 40
                    ? 'orange'
                    : 'red',
              borderRadius: '4px',
              transition: 'width 0.3s',
            }}
          />
        </div>
        <p>Password strength: {passwordStrength}%</p>
      </div>

      <div>
        <label>Confirm Password</label>
        <Input
          type="password"
          placeholder="Confirm your password"
          value={confirmPassword}
          onChange={handleConfirmPasswordChange}
        />
        {showWarning && <p style={{ color: 'red' }}>Passwords do not match!</p>}
      </div>

      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <input
            type="checkbox"
            id="createGroup"
            checked={createGroup}
            onChange={handleCreateGroupChange}
          />
          <label htmlFor="createGroup">Create the first chat group</label>
        </div>
      </div>

      {createGroup && (
        <div>
          <label>First Chat Group Name</label>
          <Input
            type="text"
            placeholder="Enter your group name"
            value={groupName}
            onChange={handleGroupNameChange}
          />
        </div>
      )}
    </div>
  )
}
