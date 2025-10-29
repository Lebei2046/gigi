import { useSignupContext } from '../context/SignupContext';
import { Button } from "@/components/ui/button"

export default function StepNavigation() {
  const { state: { currentStep, steps }, dispatch } = useSignupContext();

  return (
    <div>
      <Button
        onClick={() => dispatch({ type: "GO_TO_PREV_STEP" })}
      >
        Back
      </Button>
      <Button
        onClick={() => dispatch({ type: "GO_TO_NEXT_STEP" })}
        disabled={!steps[currentStep]}
      >
        Next
      </Button>
    </div>
  );
}