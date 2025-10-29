import { useSignupContext } from '../context/SignupContext';

interface AgreetoContinueProps {
  id: string;
  label: string;
  step: number;
  disabled?: boolean;
}

export default function AgreeToContinue({
  id,
  label,
  step,
  disabled = false,
}: AgreetoContinueProps) {
  const { dispatch, state: { steps } } = useSignupContext();

  const handleOnChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    dispatch({ type: "SET_STEP_CHECKED", payload: { index: step, checked: e.target.checked } });
  };

  return (
    <div className="flex items-center mb-6">
      <input
        type="checkbox"
        id={id}
        className="mr-2"
        checked={steps[step]}
        disabled={disabled}
        onChange={handleOnChange}
      />
      <label htmlFor={id}>{label}</label>
    </div>
  );
}
