import { createSlice, type PayloadAction } from '@reduxjs/toolkit';
import { getStorageItem } from '../utils/storage';
import { decryptMnemonics, generateAddress } from '../utils/crypto';

type AuthState = {
  status: 'unregistered' | 'unauthenticated' | 'authenticated';
  mnemonic: string | null;
  nonce: string | null;
  address: string | null;
  error: string | null;
};

const initialState: AuthState = {
  status: 'unregistered',
  mnemonic: null,
  nonce: null,
  address: null,
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
    initAuth: (state) => {
      const gigiData = getStorageItem<{ mnemonic?: string; nonce?: string; address?: string }>('gigi');
      if (!gigiData) {
        state.status = 'unregistered';
      } else {
        state.mnemonic = gigiData.mnemonic || null;
        state.nonce = gigiData.nonce || null;
        state.address = gigiData.address || null;
        state.status = 'unauthenticated';
      }
    },
    login: (state, action: PayloadAction<{ password: string }>) => {
      if (!state.mnemonic || !state.nonce || !state.address) {
        return;
      }
      const { password } = action.payload;
      try {
        const decryptedMnemonics = decryptMnemonics(state.mnemonic, password, state.nonce);
        const generatedAddress = generateAddress(decryptedMnemonics);
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
  },
});

export const { clearAuth, setUnregistered, initAuth, login } = authSlice.actions;
export default authSlice.reducer;