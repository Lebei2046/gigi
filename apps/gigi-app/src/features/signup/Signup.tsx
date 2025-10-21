import type { JSX } from 'react';
import {
  SignupProvider,
  useSignupContext
} from "./context/SignupContext";
import StepNavigation from "./components/StepNavigation";
import TermsOfUse from "./components/TermsOfUse";
import MnemonicDisplay from "./components/MnemonicDisplay";
import MnemonicInput from "./components/MnemonicInput";
import MnemonicConfirm from "./components/MnemonicConfirm";
import SignupInfoInput from "./components/SignupInfoInput";
import Welcome from "./pages/Welcome";
import SignupFinish from "./pages/SignupFinish";

export default function Signup() {
  return (
    <SignupProvider>
      <SignupContent />
    </SignupProvider>
  );
}

function SignupContent() {
  const { state: { signupType, currentStep } } = useSignupContext();

  const FINISH_STEP: number = 4;
  const STEPS: Array<{ component: JSX.Element; label: string }> = [
    { component: <TermsOfUse />, label: "Terms of Use" },
    { component: signupType === "create" ? <MnemonicDisplay /> : <MnemonicInput />, label: "Mnemonic Input/Display" },
    { component: <MnemonicConfirm />, label: "Confirm Mnemonic" },
    { component: <SignupInfoInput />, label: "Signup Info" },
  ];

  if (signupType === null) {
    return <Welcome />;
  } else if (currentStep === FINISH_STEP) {
    return <SignupFinish />;
  } else {
    return <Stepper steps={STEPS} step={currentStep} />;
  }
}

interface StepperProps {
  steps: Array<{ component: JSX.Element; label: string }>;
  step: number;
}

function Stepper({ steps, step }: StepperProps) {
  return (
    <>
      {steps[step].component}
      <StepNavigation />
    </>
  )
}