/* eslint-disable @typescript-eslint/no-explicit-any */
import { createSlice, type PayloadAction } from '@reduxjs/toolkit'
import { getStorageItem, clearStorageItem } from '../utils/settingStorage'
import {
  decryptMnemonics,
  getAddress,
  getPrivateKeyFromMnemonic,
} from '../utils/crypto'
import { MessagingClient } from '../utils/messaging'

type AuthState = {
  status: 'unregistered' | 'unauthenticated' | 'authenticated'
  mnemonic: string | null
  nonce: string | null
  address: string | null
  peerId: string | null
  name: string | null
  error: string | null
}

const initialState: AuthState = {
  status: 'unregistered',
  mnemonic: null,
  nonce: null,
  address: null,
  peerId: null,
  name: null,
  error: null,
}

const authSlice = createSlice({
  name: 'auth',
  initialState,
  reducers: {
    clearAuth: state => {
      state.status = 'unauthenticated'
    },
    setUnregistered: state => {
      state.status = 'unregistered'
    },
    login: (state, action: PayloadAction<{ password: string }>) => {
      if (!state.mnemonic || !state.nonce || !state.address) {
        return
      }
      const { password } = action.payload
      try {
        const decryptedMnemonics = decryptMnemonics(
          state.mnemonic,
          password,
          state.nonce
        )
        const generatedAddress = getAddress(decryptedMnemonics)
        if (generatedAddress === state.address) {
          state.status = 'authenticated'
          state.error = null
        } else {
          state.error = 'Password is incorrect, please re-enter!'
        }
      } catch (error) {
        state.error =
          error instanceof Error
            ? error.message
            : 'Decryption failed, please check if data or password is correct'
      }
    },
    resetState: state => {
      state.status = 'unregistered'
      state.mnemonic = null
      state.nonce = null
      state.address = null
      state.peerId = null
      state.name = null
      state.error = null
    },
    setError: (state, action: PayloadAction<string>) => {
      state.error = action.payload
    },
  },
  extraReducers: builder => {
    builder.addCase(
      'auth/initAuth/fulfilled' as any,
      (state, action: PayloadAction<any>) => {
        const gigiData = action.payload
        state.mnemonic = gigiData.mnemonic || null
        state.nonce = gigiData.nonce || null
        state.address = gigiData.address || null
        state.peerId = gigiData.peerId || null
        state.name = gigiData.name || null
        state.status = 'unauthenticated'
      }
    )
  },
})

// Async action to load auth data from IndexedDB
export const loadAuthData = () => async (dispatch: any) => {
  try {
    const gigiData = await getStorageItem<{
      mnemonic?: string
      nonce?: string
      address?: string
      peerId?: string
      name?: string
    }>('gigi')

    if (!gigiData) {
      dispatch(setUnregistered())
    } else {
      dispatch({
        type: 'auth/initAuth/fulfilled',
        payload: gigiData,
      })
    }
  } catch (error) {
    console.error('Failed to load auth data:', error)
    dispatch(setUnregistered())
  }
}

// Async action to reset auth data
export const resetAuth = () => async (dispatch: any) => {
  try {
    await clearStorageItem('gigi')
    dispatch(resetState())
  } catch (error) {
    console.error('Failed to reset auth data:', error)
    dispatch(resetState()) // Still reset state even if storage clear fails
  }
}

// Async action for login with P2P initialization
export const loginWithP2P =
  (password: string) => async (dispatch: any, getState: any) => {
    const state = getState().auth
    if (!state.mnemonic || !state.nonce || !state.address) {
      return { success: false, error: 'No auth data available' }
    }

    try {
      const decryptedMnemonics = decryptMnemonics(
        state.mnemonic,
        password,
        state.nonce
      )
      const generatedAddress = getAddress(decryptedMnemonics)

      if (generatedAddress !== state.address) {
        return {
          success: false,
          error: 'Password is incorrect, please re-enter!',
        }
      }

      // Extract private key and initialize P2P
      const privateKey = getPrivateKeyFromMnemonic(decryptedMnemonics)
      // Use the stored name as nickname, fallback to "Anonymous" if not available
      const nickname = state.name || 'Anonymous'
      const peerId = await MessagingClient.initializeWithKey(
        privateKey,
        nickname
      )

      dispatch(login({ password }))

      return { success: true, peerId }
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : 'P2P initialization failed'
      return { success: false, error: errorMessage }
    }
  }

export const { clearAuth, setUnregistered, login, resetState, setError } =
  authSlice.actions
export default authSlice.reducer
