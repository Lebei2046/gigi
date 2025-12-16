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
    <div className="space-y-6">
      <div className="text-center space-y-2">
        <h1 className="text-2xl font-bold text-gray-900">Create Account</h1>
        <p className="text-gray-600">
          Set up your account password and preferences
        </p>
      </div>

      <div className="space-y-4">
        <div className="space-y-2">
          <label className="text-sm font-medium text-gray-700">
            Account Name
          </label>
          <Input
            type="text"
            placeholder="Enter your account name"
            value={name}
            onChange={handleNameChange}
            className="w-full"
          />
        </div>

        <div className="space-y-2">
          <label className="text-sm font-medium text-gray-700">Password</label>
          <Input
            type="password"
            placeholder="Enter your password"
            value={password}
            onChange={handlePasswordChange}
            className="w-full"
          />
          <div className="space-y-1">
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div
                className={`h-2 rounded-full transition-all duration-300 ${
                  passwordStrength > 70
                    ? 'bg-green-500'
                    : passwordStrength > 40
                      ? 'bg-yellow-500'
                      : 'bg-red-500'
                }`}
                style={{ width: `${passwordStrength}%` }}
              />
            </div>
            <p className="text-xs text-gray-600">
              Password strength:
              <span
                className={`font-medium ${
                  passwordStrength > 70
                    ? 'text-green-600'
                    : passwordStrength > 40
                      ? 'text-yellow-600'
                      : 'text-red-600'
                }`}
              >
                {passwordStrength > 70
                  ? ' Strong'
                  : passwordStrength > 40
                    ? ' Medium'
                    : ' Weak'}
              </span>
            </p>
          </div>
        </div>

        <div className="space-y-2">
          <label className="text-sm font-medium text-gray-700">
            Confirm Password
          </label>
          <Input
            type="password"
            placeholder="Confirm your password"
            value={confirmPassword}
            onChange={handleConfirmPasswordChange}
            className={`w-full ${showWarning ? 'border-red-300' : ''}`}
          />
          {showWarning && (
            <p className="text-sm text-red-600 flex items-center gap-1">
              <span>⚠️</span> Passwords do not match!
            </p>
          )}
        </div>

        <div className="bg-gray-50 rounded-lg p-4 border border-gray-200">
          <div className="flex items-center space-x-3">
            <input
              type="checkbox"
              id="createGroup"
              checked={createGroup}
              onChange={handleCreateGroupChange}
              className="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 focus:ring-2"
            />
            <label
              htmlFor="createGroup"
              className="text-sm font-medium text-gray-700"
            >
              Create the first chat group
            </label>
          </div>

          {createGroup && (
            <div className="mt-4 space-y-2">
              <label className="text-sm font-medium text-gray-700">
                Group Name
              </label>
              <Input
                type="text"
                placeholder="Enter your group name"
                value={groupName}
                onChange={handleGroupNameChange}
                className="w-full"
              />
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
