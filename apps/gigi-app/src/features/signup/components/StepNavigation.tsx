import { useSignupContext } from '../context/SignupContext';

const StepNavigation = () => {
  const { goToNextStep, goToPrevStep, isNextDisabled } = useSignupContext();

  return (
    <div className="flex justify-between mt-6">
      <button
        className="btn btn-outline w-40"
        onClick={goToPrevStep}
      >
        Back
      </button>
      <button
        className="btn btn-primary w-40"
        onClick={goToNextStep}
        disabled={isNextDisabled}
      >
        Next
      </button>
    </div>
  );
};

export default StepNavigation;