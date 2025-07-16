import React, { createContext, useContext, useState } from "react";

type SignupType = "create" | "import" | null;

type SignupContextType = {
  currentStep: number;
  goToNextStep: () => void;
  goToPrevStep: () => void;
  isNextDisabled: boolean;
  setIsNextDisabled: (disabled: boolean) => void;
  signupType: SignupType;
  setSignupType: (type: SignupType) => void;
  mnemonic: string[];
  setMnemonic: (mnemonic: string[]) => void;
  password: string;
  setPassword: (password: string) => void;
};

const SignupContext = createContext<SignupContextType | undefined>(undefined);

export const SignupProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [isNextDisabled, setIsNextDisabled] = useState(false);
  const [signupType, setSignupType] = useState<SignupType>(null);
  const [mnemonic, setMnemonic] = useState<string[]>(Array(12).fill(""));
  const [password, setPassword] = useState("");

  const goToNextStep = () => {
    if (!isNextDisabled) {
      setCurrentStep((prev) => prev + 1);
    }
  };

  const goToPrevStep = () => {
    setCurrentStep((prev) => Math.max(0, prev - 1));

    if (currentStep === 0) {
      setSignupType(null);
    }
  };

  return (
    <SignupContext.Provider
      value={{
        currentStep,
        goToNextStep,
        goToPrevStep,
        isNextDisabled,
        setIsNextDisabled,
        signupType,
        setSignupType,
        mnemonic,
        setMnemonic,
        password,
        setPassword,
      }}
    >
      {children}
    </SignupContext.Provider>
  );
};

// eslint-disable-next-line react-refresh/only-export-components
export const useSignupContext = () => {
  const context = useContext(SignupContext);
  if (!context) {
    throw new Error("useSignupContext must be used within a SignupProvider");
  }
  return context;
};