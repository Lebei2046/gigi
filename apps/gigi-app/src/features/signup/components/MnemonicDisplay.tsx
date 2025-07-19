import { useEffect } from "react";
import { useSignupContext } from "../context/SignupContext";
import { generateMnemonics } from "../../../utils/crypto";
import AgreeToContinue from "./AgreeToContinue";
import { FaCopy, FaExclamationTriangle } from "react-icons/fa";

export default function MnemonicDisplay() {
  const STEP: number = 1;

  const { state: { mnemonic }, dispatch } = useSignupContext();

  useEffect(() => {
    const generatedMnemonic = generateMnemonics();
    dispatch({ type: "SET_MNEMONIC", payload: generatedMnemonic });
  }, [dispatch]);

  return (
    <div className="p-8">
      <div>
        <h1 className="text-2xl font-bold mb-4">Seed Phrase</h1>
        <p className="mb-4 text-gray-600">
          Please write down your seed phrase in the correct order and keep it in a safe place.
        </p>
      </div>
      <div className="mb-6">
        <div className="grid grid-cols-3 gap-4">
          {mnemonic.map((word: string, index: number) => (
            <div key={index} className="border rounded-lg p-2 text-center">
              <span className="text-gray-500">{index + 1}.</span>
              <span className="ml-2 font-medium">{word}</span>
            </div>
          ))}
        </div>
        <div className="flex justify-center mt-4">
          <button
            className="btn btn-outline flex items-center gap-2"
            onClick={() => {
              const mnemonicString = mnemonic.join(' ');
              navigator.clipboard.writeText(mnemonicString).then(() => {
                alert('Copied to clipboard!');
              }).catch(err => {
                console.error('Failed to copy:', err);
              });
            }}
          >
            <FaCopy className="h-5 w-5" />
            Copy
          </button>
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
      />
    </div>
  );
}
