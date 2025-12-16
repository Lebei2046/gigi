import { useSignupContext } from '../context/SignupContext'

interface AgreetoContinueProps {
  id: string
  label: string
  step: number
  disabled?: boolean
}

export default function AgreeToContinue({
  id,
  label,
  step,
  disabled = false,
}: AgreetoContinueProps) {
  const {
    dispatch,
    state: { steps },
  } = useSignupContext()

  const handleOnChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    dispatch({
      type: 'SET_STEP_CHECKED',
      payload: { index: step, checked: e.target.checked },
    })
  }

  return (
    <div className="flex items-start space-x-3 p-4 bg-gray-50 rounded-lg border border-gray-200">
      <input
        type="checkbox"
        id={id}
        className="mt-1 w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 focus:ring-2"
        checked={steps[step]}
        disabled={disabled}
        onChange={handleOnChange}
      />
      <label
        htmlFor={id}
        className="text-sm font-medium text-gray-700 leading-relaxed"
      >
        {label}
      </label>
    </div>
  )
}
