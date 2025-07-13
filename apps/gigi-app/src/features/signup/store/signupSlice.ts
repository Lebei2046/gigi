import { createSlice } from '@reduxjs/toolkit';
import type { PayloadAction } from '@reduxjs/toolkit';
import { encryptAndStore } from '../utils/db';
import { CryptoService } from '../services/crypto';

type Account = {
  id: string;
  publicKey: Uint8Array;
  address: string;
};

type SignupStep = 'terms' | 'mnemonic' | 'confirm' | 'password' | 'complete';

interface SignupState {
  accounts: Account[];
  currentAccount: Account | null;
  isLocked: boolean;
  signUp: {
    step: SignupStep;
    mnemonic: string;
    confirmed: boolean;
    password: string;
  };
}

const initialState: SignupState = {
  accounts: [],
  currentAccount: null,
  isLocked: true,
  signUp: {
    step: 'terms',
    mnemonic: '',
    confirmed: false,
    password: '',
  },
};

export const signupSlice = createSlice({
  name: 'auth',
  initialState,
  reducers: {
    nextStep: (state) => {
      const steps: SignupStep[] = ['terms', 'mnemonic', 'confirm', 'password', 'complete'];
      const currentIndex = steps.indexOf(state.signUp.step);
      if (currentIndex < steps.length - 1) {
        state.signUp.step = steps[currentIndex + 1];
      }
    },
    prevStep: (state) => {
      const steps: SignupStep[] = ['terms', 'mnemonic', 'confirm', 'password', 'complete'];
      const currentIndex = steps.indexOf(state.signUp.step);
      if (currentIndex > 0) {
        state.signUp.step = steps[currentIndex - 1];
      }
    },
    setMnemonic: (state, action: PayloadAction<string>) => {
      state.signUp.mnemonic = action.payload;
    },
    confirmMnemonic: (state) => {
      state.signUp.confirmed = true;
    },
    setPassword: (state, action: PayloadAction<string>) => {
      state.signUp.password = action.payload;
    },
    createAccount: (state, action: PayloadAction<{ password: string }>) => {
      const mnemonic = CryptoService.generateMnemonic();
      const { publicKey, privateKey } = CryptoService.deriveKeys(mnemonic);
      const accountId = crypto.randomUUID();

      encryptAndStore(
        { id: accountId, value: privateKey },
        action.payload.password
      );

      const newAccount = {
        id: accountId,
        publicKey,
        address: '', // TODO: Derive address from publicKey
      };

      state.accounts.push(newAccount);
      state.currentAccount = newAccount;
      state.isLocked = false;
    },
    unlockAccount: (state, action: PayloadAction<{ password: string }>) => {
      // TODO: Implement decryption
      state.isLocked = false;
    },
  },
});

export const {
  createAccount,
  unlockAccount,
  nextStep,
  prevStep,
  setMnemonic,
  confirmMnemonic,
  setPassword,
} = signupSlice.actions;
export default signupSlice.reducer;
