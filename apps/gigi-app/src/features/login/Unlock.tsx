import { useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { FaLock } from 'react-icons/fa';
import { login } from '../../store/authSlice';
import type { RootState } from '../../store';

export default function Unlock() {
  const dispatch = useDispatch();
  const [password, setPassword] = useState('');
  const { error } = useSelector((state: RootState) => state.auth);

  const handleLogin = () => {
    dispatch(login({ password }));
  };

  return (
    <div className="flex flex-col items-center justify-center min-h-screen p-4 bg-gray-50">
      <div className="w-full max-w-md p-8 bg-white rounded-lg shadow-md">
        <div className="flex flex-col items-center mb-6">
          <FaLock className="w-12 h-12 text-blue-500 mb-2" />
          <h1 className="text-2xl font-bold text-gray-800">Unlock Account</h1>
          <p className="text-gray-600 mt-2">Enter your password to continue</p>
        </div>

        <div className="mb-4">
          <label htmlFor="password" className="block text-sm font-medium text-gray-700 mb-1">
            Password
          </label>
          <input
            type="password"
            id="password"
            className="w-full px-4 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="Enter your password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
          {error && <p className="mt-2 text-sm text-red-500">{error}</p>}
        </div>

        <div className="flex justify-end mb-6">
          <a href="#" className="text-sm text-blue-500 hover:underline">
            Forgot password?
          </a>
        </div>

        <button
          type="button"
          className="w-full bg-blue-500 text-white py-2 px-4 rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
          onClick={handleLogin}
        >
          Unlock
        </button>
      </div>
    </div>
  );
}
