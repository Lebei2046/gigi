/* eslint-disable @typescript-eslint/no-explicit-any */
import { createSlice, type PayloadAction } from '@reduxjs/toolkit'
import {
  authCheckAccount,
  authGetAccountInfo,
  authLoginWithP2P,
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

// Async action for login with P2P initialization (combined command)
export const loginWithP2P =
  (password: string) => async (dispatch: any, getState: any) => {
    const state = getState().auth

    try {
      const accountInfo = await authLoginWithP2P(password)

      // Use the generated action creator to update Redux state
      dispatch(
        login({
          address: accountInfo.address,
          peerId: accountInfo.peer_id,
          groupId: accountInfo.group_id,
          name: accountInfo.name,
        })
      )

      return { success: true, peerId: accountInfo.peer_id }
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
