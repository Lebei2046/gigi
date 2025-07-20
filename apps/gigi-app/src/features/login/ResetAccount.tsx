import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { FaTimes } from 'react-icons/fa';
import { useAppDispatch } from '../../store';
import { reset } from '../../store/authSlice';

export default function ResetAccount() {
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const [checked, setChecked] = useState(false);

  return (
    <div className="flex flex-col min-h-screen p-4">
      {/* Navigation Bar */}
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-xl font-bold">Forgot Password</h1>
        <button
          onClick={() => navigate(-1)}
          className="w-8 h-8 flex items-center justify-center bg-gray-200 rounded-full hover:bg-gray-300"
        >
          <FaTimes className="h-4 w-4" />
        </button>
      </div>

      {/* Warning Text */}
      <div className="mb-6">
        <p className="mb-4">
          Resetting your account will permanently delete all your data, including your wallet and transaction history.
        </p>
        <p className="mb-4">
          This action cannot be undone. Please ensure you have backed up your recovery phrase before proceeding.
        </p>
      </div>

      {/* Warning Card */}
      <div className="bg-yellow-100 border-l-4 border-yellow-500 p-4 mb-6">
        <p className="text-yellow-700">
          <strong>Warning:</strong> This is a destructive action. Proceed only if you understand the consequences.
        </p>
      </div>

      {/* Checkbox */}
      <div className="flex items-center mb-6">
        <input
          type="checkbox"
          id="accept-risk"
          checked={checked}
          onChange={() => setChecked(!checked)}
          className="mr-2"
        />
        <label htmlFor="accept-risk">I understand and accept the risks</label>
      </div>

      {/* Reset Button */}
      <button
        disabled={!checked}
        className={`px-4 py-2 rounded-md ${checked ? 'bg-red-500 hover:bg-red-600 text-white' : 'bg-gray-300 cursor-not-allowed'}`}
        onClick={() => {
          dispatch(reset());
          navigate('/signup');
        }}
      >
        Reset Account
      </button>
    </div>
  );
}