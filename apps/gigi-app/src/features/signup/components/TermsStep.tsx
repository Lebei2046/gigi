import { useAppDispatch } from '@/store';
import { nextStep } from '../store/signupSlice';

export function TermsStep() {
  const dispatch = useAppDispatch();

  return (
    <div>
      <h2>Terms of Use</h2>
      <p>Please read and accept the terms to continue.</p>
      <button onClick={() => dispatch(nextStep())}>Accept</button>
    </div>
  );
}
