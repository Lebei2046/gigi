import { useSignupContext } from '../context/SignupContext';

export default function StepNavigation() {
  const { state: { currentStep, steps }, dispatch } = useSignupContext();

  return (
    <div className="flex justify-between mt-6">
      <button
        className="btn btn-outline w-40"
        onClick={() => dispatch({ type: "GO_TO_PREV_STEP" })}
      >
        Back
      </button>
      <button
        className="btn btn-primary w-40"
        onClick={() => dispatch({ type: "GO_TO_NEXT_STEP" })}
        disabled={!steps[currentStep]}
      >
        Next
      </button>
    </div>
  );
}
