import React, { useState } from 'react';

export default function PasswordInput() {
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [passwordStrength, setPasswordStrength] = useState(0);

  const handlePasswordChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setPassword(value);
    // Calculate password strength (0-100)
    const strength = Math.min(value.length * 10, 100);
    setPasswordStrength(strength);
  };

  const handleConfirmPasswordChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setConfirmPassword(e.target.value);
  };

  return (
    <div className="p-8 bg-white">
      <div>
        <h1 className="text-2xl font-bold mb-4">Create password for encryption</h1>
        <p className="mb-4 text-gray-600">
          This password will be used to unlock your wallet and account.
        </p>
      </div>

      <div className="mb-6">
        <div className="mb-4">
          <label className="block text-gray-700 mb-2">Password</label>
          <input
            type="password"
            className="w-full px-4 py-2 border border-gray-300 rounded"
            placeholder="Enter your password"
            value={password}
            onChange={handlePasswordChange}
          />
          <div className="mt-2 h-2 bg-gray-200 rounded">
            <div
              className="h-2 rounded bg-green-500"
              style={{ width: `${passwordStrength}%` }}
            ></div>
          </div>
          <p className="text-xs text-gray-500 mt-1">
            Password strength: {passwordStrength}%
          </p>
        </div>

        <div className="mb-6">
          <label className="block text-gray-700 mb-2">Confirm Password</label>
          <input
            type="password"
            className="w-full px-4 py-2 border border-gray-300 rounded"
            placeholder="Confirm your password"
            value={confirmPassword}
            onChange={handleConfirmPasswordChange}
          />
        </div>
      </div>
    </div>
  );
}
