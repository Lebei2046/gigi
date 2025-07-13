import { useState } from 'react';
import { TermsStep } from './TermsStep';
import { MnemonicStep } from './MnemonicStep';
import { ConfirmStep } from './ConfirmStep';
import { PasswordStep } from './PasswordStep';
import { CompleteStep } from './CompleteStep';

export default function SignupPanel() {
  const [currentStep, setCurrentStep] = useState(0);

  const steps = [
    <TermsStep key="terms" />,
    <MnemonicStep key="mnemonic" />,
    <ConfirmStep key="confirm" />,
    <PasswordStep key="password" />,
    <CompleteStep key="complete" />,
  ];

  return (
    <div>
      {steps[currentStep]}
      <div>
        {currentStep > 0 && (
          <button onClick={() => setCurrentStep(currentStep - 1)}>Back</button>
        )}
        {currentStep < steps.length - 1 && (
          <button onClick={() => setCurrentStep(currentStep + 1)}>Next</button>
        )}
      </div>
    </div>
  );
}
