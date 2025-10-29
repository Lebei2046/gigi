import React, {
  createContext,
  useContext,
  useReducer
} from "react";
import {
  initialState,
  signupReducer,
  type SignupAction,
  type SignupType
} from "./signupReducer";
import { encryptMnemonics, generateAddress } from "@/utils/crypto";
import { setStorageItem } from "@/utils/settingStorage";

type SignupContextType = {
  state: {
    currentStep: number;
    steps: boolean[];
    signupType: SignupType;
    mnemonic: string[];
    password: string;
    address: string;
    name: string;
  };
  dispatch: React.Dispatch<SignupAction>;
  saveAccountInfo: () => Promise<void>;
};

const SignupContext = createContext<SignupContextType | undefined>(undefined);

export function SignupProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(signupReducer, initialState);

  const saveAccountInfo = async () => {
    const walletAddress = generateAddress(state.mnemonic);
    const { mnemonic: cryptedMnemonic, nonce } = encryptMnemonics(
      state.mnemonic,
      state.password
    );

    // Save to IndexedDB
    await setStorageItem("gigi", {
      nonce,
      mnemonic: cryptedMnemonic,
      address: walletAddress,
      name: state.name,
    });

    // Update state after successful save
    dispatch({ type: "ACCOUNT_INFO_SAVED", payload: { address: walletAddress } });
  };

  return (
    <SignupContext.Provider value={{ state, dispatch, saveAccountInfo }}>
      {children}
    </SignupContext.Provider>
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export function useSignupContext() {
  const context = useContext(SignupContext);
  if (!context) {
    throw new Error("useSignupContext must be used within a SignupProvider");
  }
  return context;
}
