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
};

const SignupContext = createContext<SignupContextType | undefined>(undefined);

export function SignupProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(signupReducer, initialState);

  return (
    <SignupContext.Provider value={{ state, dispatch }}>
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