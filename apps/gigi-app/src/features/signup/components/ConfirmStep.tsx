import { useAppDispatch, useAppSelector } from '@/store';
import { nextStep, confirmMnemonic } from '../store/signupSlice';

interface SignUpState {
  mnemonic: string;
  // add other signUp properties if needed
}

interface SignupState {
  signUp: SignUpState;
  // add other auth properties if needed
}

export function ConfirmStep() {
  const dispatch = useAppDispatch();
  const { mnemonic } = useAppSelector((state: { signup: SignupState }) => state.signup.signUp);

  return (
    <div>
      <h2>Confirm Your Mnemonic</h2>
      <p>Enter your mnemonic phrase to confirm:</p>
      <input type="text" placeholder="Enter mnemonic" />
      <button onClick={() => {
        dispatch(confirmMnemonic());
        dispatch(nextStep());
      }}>Confirm</button>
    </div>
  );
}
