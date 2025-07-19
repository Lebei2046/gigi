import { useEffect, useState } from "react";
import { useSignupContext } from "../context/SignupContext";
import AgreeToContinue from "./AgreeToContinue";
import { FaExclamationTriangle } from "react-icons/fa";

export default function MnemonicInput() {
  const STEP: number = 1;

  const { state: { mnemonic }, dispatch } = useSignupContext();
  const [isCheckboxDisabled, setIsCheckboxDisabled] = useState<boolean>(true);

  const handleChange = (index: number, value: string) => {
    const newMnemonic = [...mnemonic];
    newMnemonic[index] = value;
    dispatch({ type: "SET_MNEMONIC", payload: newMnemonic });
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
            <FaExclamationTriangle className="h-5 w-5 text-yellow-400" />
          </div>
          <div className="ml-3">
            <p className="text-sm text-yellow-700">
              Anyone with access to your recovery phrase can access your assets. Store it securely. Gigi does not keep a backup of your 12-word phrase.
            </p>
          </div>
        </div>
      </div>
      <AgreeToContinue
        id="seedPhraseConfirmation"
        label="I have written down my seed phrase on paper and stored it securely"
        step={STEP}
        disabled={isCheckboxDisabled}
      />
    </div>
  );
}
