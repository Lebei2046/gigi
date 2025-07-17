import { useSignupContext } from '../context/SignupContext';

interface AgreetoContinueProps {
  id: string;
  label: string;
  disabled?: boolean;
}

export default function AgreeToContinue({
  id,
  label,
  disabled = false,
}: AgreetoContinueProps) {
  const { dispatch } = useSignupContext();

  const handleOnChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    dispatch({ type: "SET_IS_NEXT_DISABLED", payload: !e.target.checked });
  };

  return (
    <div className="flex items-center mb-6">
      <input
        type="checkbox"
        id={id}
        className="mr-2"
        disabled={disabled}
        onChange={handleOnChange}
      />
      <label htmlFor={id}>{label}</label>
    </div>
  );
}
