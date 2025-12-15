import React, { createContext, useContext, useReducer } from 'react'
import {
  initialState,
  signupReducer,
  type SignupAction,
  type SignupState,
} from './signupReducer'
import {
  encryptMnemonics,
  generateAddress,
  generateGroupPeerId,
} from '@/utils/crypto'
import { setStorageItem } from '@/utils/settingStorage'
import { db } from '@/models/db'

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
    const { address, peerId } = await generateAddress(state.mnemonic)
    const { mnemonic: cryptedMnemonic, nonce } = encryptMnemonics(
      state.mnemonic,
      state.password
    )

    // Save to IndexedDB
    await setStorageItem('gigi', {
      nonce,
      mnemonic: cryptedMnemonic,
      address,
      peerId,
      name: state.name,
    })

    // Update state after successful save
    dispatch({ type: 'ACCOUNT_INFO_SAVED', payload: { address, peerId } })
  }

  const saveGroupInfo = async () => {
    if (!state.createGroup || !state.groupName.trim()) {
      return
    }

    try {
      // Derive group peer ID from user's mnemonic
      const groupPeerId = await generateGroupPeerId(state.mnemonic)

      // Save group to IndexedDB
      await db.groups.add({
        id: groupPeerId,
        name: state.groupName.trim(),
        joined: false, // User hasn't joined yet, just created it
        createdAt: new Date(),
      })

      console.log('Group saved successfully:', {
        id: groupPeerId,
        name: state.groupName.trim(),
        joined: false,
        createdAt: new Date(),
      })
    } catch (error) {
      console.error('Failed to save group:', error)
      throw error
    }
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
