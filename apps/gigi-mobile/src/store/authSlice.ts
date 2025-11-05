/* eslint-disable @typescript-eslint/no-explicit-any */
import { createSlice, type PayloadAction } from '@reduxjs/toolkit';
import { getStorageItem, clearStorageItem } from '../utils/settingStorage';
import { decryptMnemonics, getAddress } from '../utils/crypto';

type AuthState = {
  status: 'unregistered' | 'unauthenticated' | 'authenticated';
  mnemonic: string | null;
  nonce: string | null;
  address: string | null;
  peerId: string | null;
  name: string | null;
  error: string | null;
};

const initialState: AuthState = {
  status: 'unregistered',
  mnemonic: null,
  nonce: null,
  address: null,
  peerId: null,
  name: null,
  error: null,
};

const authSlice = createSlice({
  name: 'auth',
  initialState,
  reducers: {
    clearAuth: (state) => {
      state.status = 'unauthenticated';
    },
    setUnregistered: (state) => {
      state.status = 'unregistered';
    },
    login: (state, action: PayloadAction<{ password: string }>) => {
      if (!state.mnemonic || !state.nonce || !state.address) {
        return;
      }
      const { password } = action.payload;
      try {
        const decryptedMnemonics = decryptMnemonics(state.mnemonic, password, state.nonce);
        const generatedAddress = getAddress(decryptedMnemonics);
        if (generatedAddress === state.address) {
          state.status = 'authenticated';
          state.error = null;
        } else {
          state.error = '密码有误，请重新输入！';
        }
      } catch (error) {
        state.error = error instanceof Error ? error.message : '解密失败，请检查数据或密码是否正确';
      }
    },
    resetState: (state) => {
      state.status = 'unregistered';
      state.mnemonic = null;
      state.nonce = null;
      state.address = null;
      state.peerId = null;
      state.name = null;
      state.error = null;
    },
  },
  extraReducers: (builder) => {
    builder.addCase('auth/initAuth/fulfilled' as any, (state, action: PayloadAction<any>) => {
      const gigiData = action.payload;
      state.mnemonic = gigiData.mnemonic || null;
      state.nonce = gigiData.nonce || null;
      state.address = gigiData.address || null;
      state.peerId = gigiData.peerId || null;
      state.name = gigiData.name || null;
      state.status = 'unauthenticated';
    });
  }
});

// Async action to load auth data from IndexedDB
export const loadAuthData = () => async (dispatch: any) => {
  try {
    const gigiData = await getStorageItem<{
      mnemonic?: string;
      nonce?: string;
      address?: string;
      peerId?: string;
      name?: string
    }>('gigi');

    if (!gigiData) {
      dispatch(setUnregistered());
    } else {
      dispatch({
        type: 'auth/initAuth/fulfilled',
        payload: gigiData
      });
    }
  } catch (error) {
    console.error('Failed to load auth data:', error);
    dispatch(setUnregistered());
  }
};

// Async action to reset auth data
export const resetAuth = () => async (dispatch: any) => {
  try {
    await clearStorageItem('gigi');
    dispatch(resetState());
  } catch (error) {
    console.error('Failed to reset auth data:', error);
    dispatch(resetState()); // Still reset state even if storage clear fails
  }
};

export const { clearAuth, setUnregistered, login, resetState } = authSlice.actions;
export default authSlice.reducer;
