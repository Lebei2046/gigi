import { useState } from 'react';
import { useAppDispatch } from '@/store';
import { nextStep, setPassword as setPasswordAction } from '../store/signupSlice';

export function PasswordStep() {
  const dispatch = useAppDispatch();
  const [password, setPassword] = useState('');

  return (
    <div>
      <h2>Set a Password</h2>
      <input
        type="password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        placeholder="Enter password"
      />
      <button onClick={() => {
        dispatch(setPasswordAction(password));
        dispatch(nextStep());
      }}>Continue</button>
    </div>
  );
}
