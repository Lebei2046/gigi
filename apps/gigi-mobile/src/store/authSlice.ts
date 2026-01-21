/* eslint-disable @typescript-eslint/no-explicit-any */
import { createSlice, type PayloadAction } from '@reduxjs/toolkit'
import {
  authCheckAccount,
  authGetAccountInfo,
  authLogin,
  authDeleteAccount,
} from '../utils/tauriCommands'
import { MessagingClient } from '../utils/messaging'

type AuthState = {
  status: 'unregistered' | 'unauthenticated' | 'authenticated'
  address: string | null
  peerId: string | null
  groupId: string | null
  name: string | null
  error: string | null
}

const initialState: AuthState = {
  status: 'unregistered',
  address: null,
  peerId: null,
  groupId: null,
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
    login: (
      state,
      action: PayloadAction<{
        address: string
        peerId: string
        groupId: string
        name: string
      }>
    ) => {
      state.status = 'authenticated'
      state.address = action.payload.address
      state.peerId = action.payload.peerId
      state.groupId = action.payload.groupId
      state.name = action.payload.name
      state.error = null
    },
    resetState: state => {
      state.status = 'unregistered'
      state.address = null
      state.peerId = null
      state.groupId = null
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
        const accountInfo = action.payload
        state.address = accountInfo.address || null
        state.peerId = accountInfo.peer_id || null
        state.groupId = accountInfo.group_id || null
        state.name = accountInfo.name || null
        state.status = 'unauthenticated'
      }
    )
  },
})

// Async action to load auth data from backend
export const loadAuthData = () => async (dispatch: any, getState: any) => {
  try {
    const hasAccount = await authCheckAccount()

    // Don't reset status if already authenticated (prevents race condition after login)
    const currentStatus = getState().auth.status

    if (!hasAccount) {
      if (currentStatus === 'unregistered') return // Skip if already set
      dispatch(setUnregistered())
    } else {
      const accountInfo = await authGetAccountInfo()
      if (accountInfo) {
        // Only set unauthenticated if not already authenticated
        if (currentStatus !== 'authenticated') {
          dispatch({
            type: 'auth/initAuth/fulfilled',
            payload: accountInfo,
          })
        }
      } else {
        if (currentStatus === 'unregistered') return // Skip if already set
        dispatch(setUnregistered())
      }
    }
  } catch (error) {
    console.error('Failed to load auth data:', error)
    if (currentStatus !== 'authenticated') {
      // Don't override authenticated status on error
      dispatch(setUnregistered())
    }
  }
}

// Async action to reset auth data
export const resetAuth = () => async (dispatch: any) => {
  try {
    await authDeleteAccount()
    dispatch(resetState())
  } catch (error) {
    console.error('Failed to reset auth data:', error)
    dispatch(resetState()) // Still reset state even if delete fails
  }
}

// Async action for login with P2P initialization
export const loginWithP2P =
  (password: string) => async (dispatch: any, getState: any) => {
    const state = getState().auth

    try {
      const loginResult = await authLogin(password)

      // Convert hex string to Uint8Array
      const privateKeyHex = loginResult.private_key
      const privateKeyBytes = new Uint8Array(privateKeyHex.length / 2)
      for (let i = 0; i < privateKeyHex.length; i += 2) {
        privateKeyBytes[i / 2] = parseInt(privateKeyHex.substr(i, 2), 16)
      }

      // Use the account name as nickname, fallback to "Anonymous" if not available
      const nickname = loginResult.account_info.name || 'Anonymous'
      const peerId = await MessagingClient.initializeWithKey(
        privateKeyBytes,
        nickname
      )

      // Use the generated action creator instead of manually creating action object
      // This ensures the action type matches the reducer exactly
      dispatch(
        login({
          address: loginResult.account_info.address,
          peerId: loginResult.account_info.peer_id,
          groupId: loginResult.account_info.group_id,
          name: loginResult.account_info.name,
        })
      )

      return { success: true, peerId }
    } catch (error) {
      console.error('Login error:', error)
      const errorMessage =
        error instanceof Error ? error.message : 'Login failed'
      return { success: false, error: errorMessage }
    }
  }

export const { clearAuth, setUnregistered, login, resetState, setError } =
  authSlice.actions
export default authSlice.reducer
