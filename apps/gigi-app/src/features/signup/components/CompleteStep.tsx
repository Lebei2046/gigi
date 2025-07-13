import { useAppDispatch } from '@/store';
import { createAccount } from '../store/signupSlice';

export function CompleteStep() {
  const dispatch = useAppDispatch();

  return (
    <div>
      <h2>Registration Complete!</h2>
      <button onClick={() => dispatch(createAccount({ /* TODO: add required payload fields here */ }))}>Finish</button>
    </div>
  );
}
