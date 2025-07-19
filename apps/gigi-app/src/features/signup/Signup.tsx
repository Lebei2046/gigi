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

  if (signupType === null) {
    return <Welcome />;
  } else if (currentStep === 4) {
    return <SignupFinish />;
  } else {
    return <Stepper signupType={signupType} step={currentStep} />;
  }
}

interface StepperProps {
  signupType: "create" | "import";
  step: number;
}

function Stepper({ signupType, step }: StepperProps) {
  const steps = [
    { component: <TermsOfUse />, label: "Terms of Use" },
    { component: signupType === "create" ? <MnemonicDisplay /> : <MnemonicInput />, label: "Mnemonic" },
    { component: <MnemonicConfirm />, label: "Confirm Mnemonic" },
    { component: <SignupInfoInput />, label: "Password" },
  ];

  return (
    <>
      {steps[step].component}
      <StepNavigation />
    </>
  )
}