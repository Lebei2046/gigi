import React, { createContext, useContext, useReducer } from 'react'
import {
  initialState,
  signupReducer,
  type SignupAction,
  type SignupState,
} from './signupReducer'
import { authSignup } from '@/utils/tauriCommands'

type SignupContextType = {
  state: SignupState
  dispatch: React.Dispatch<SignupAction>
  saveAccountInfo: () => Promise<void>
  saveGroupInfo: () => Promise<void>
}

const SignupContext = createContext<SignupContextType | undefined>(undefined)

export function SignupProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(signupReducer, initialState)

  const saveAccountInfo = async () => {
    try {
      const accountInfo = await authSignup(
        state.mnemonic.join(' '),
        state.password,
        state.name,
        state.createGroup ? state.groupName : undefined
      )

      // Update state after successful save
      dispatch({
        type: 'ACCOUNT_INFO_SAVED',
        payload: {
          address: accountInfo.address,
          peerId: accountInfo.peer_id,
        },
      })
    } catch (error) {
      console.error('Failed to save account info:', error)
      throw error
    }
  }

  const saveGroupInfo = async () => {
    // Group is already created in saveAccountInfo if groupName was provided
    // This function is kept for compatibility but does nothing
    console.log('Group info already saved during account creation')
  }

  return (
    <SignupContext.Provider
      value={{ state, dispatch, saveAccountInfo, saveGroupInfo }}
    >
      {children}
    </SignupContext.Provider>
  )
}

// eslint-disable-next-line react-refresh/only-export-components
export function useSignupContext() {
  const context = useContext(SignupContext)
  if (!context) {
    throw new Error('useSignupContext must be used within a SignupProvider')
  }
  return context
}
