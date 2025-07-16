import { useEffect, useState } from "react";
import { useSignupContext } from "../context/SignupContext";

export default function MnemonicInput() {
  const { mnemonic, setMnemonic } = useSignupContext();
  const [isCheckboxDisabled, setIsCheckboxDisabled] = useState<boolean>(true);

  const handleChange = (index: number, value: string) => {
    const newMnemonic = [...mnemonic];
    newMnemonic[index] = value;
    setMnemonic(newMnemonic);
  };

  useEffect(() => {
    const isAllFilled = mnemonic.every(word => word.trim() !== "");
    setIsCheckboxDisabled(!isAllFilled);
  }, [mnemonic]);

  return (
    <div className="p-8">
      <div>
        <h1 className="text-2xl font-bold mb-4">Recover Account</h1>
        <p className="mb-4 text-gray-600">
          Enter your existing seed phrase to restore your wallet.
        </p>
      </div>
      <div className="mb-6">
        <div className="grid grid-cols-3 gap-4">
          {Array.from({ length: 12 }).map((_, index) => (
            <div key={index} className="flex items-end p-2">
              <span className="text-gray-500 flex items-end justify-end w-8">{index + 1}.</span>
              <input
                type="text"
                className="ml-2 font-medium border-b w-full"
                placeholder="word"
                value={mnemonic[index]}
                onChange={(e) => handleChange(index, e.target.value)}
              />
            </div>
          ))}
        </div>
      </div>
      <div className="bg-yellow-50 border-l-4 border-yellow-400 p-4 mb-6">
        <div className="flex">
          <div className="flex-shrink-0">
            <svg className="h-5 w-5 text-yellow-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
              <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
            </svg>
          </div>
          <div className="ml-3">
            <p className="text-sm text-yellow-700">
              Anyone with access to your recovery phrase can access your assets. Store it securely. Gigi does not keep a backup of your 12-word phrase.
            </p>
          </div>
        </div>
      </div>
      <div className="flex items-center mb-6">
        <input type="checkbox" id="seedPhraseConfirmation" className="mr-2" disabled={isCheckboxDisabled} />
        <label htmlFor="seedPhraseConfirmation">I have written down my seed phrase on paper and stored it securely</label>
      </div>
    </div>
  );
}
