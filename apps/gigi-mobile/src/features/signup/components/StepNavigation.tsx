import { useSignupContext } from '../context/SignupContext'
import { Button } from '@/components/ui/button'

export default function StepNavigation() {
  const {
    state: { currentStep, steps },
    dispatch,
  } = useSignupContext()

  return (
    <div className="flex justify-between items-center space-x-4 pt-6">
      <Button
        onClick={() => dispatch({ type: 'GO_TO_PREV_STEP' })}
        variant="outline"
        className="flex-1 py-3 rounded-xl font-medium border-gray-300 hover:bg-gray-50"
      >
        ← Back
      </Button>
      <Button
        onClick={() => dispatch({ type: 'GO_TO_NEXT_STEP' })}
        disabled={!steps[currentStep]}
        className="flex-1 py-3 rounded-xl font-medium bg-blue-600 hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-all duration-200"
      >
        Next →
      </Button>
    </div>
  )
}
