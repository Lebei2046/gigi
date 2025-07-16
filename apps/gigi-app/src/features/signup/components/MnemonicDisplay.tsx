import { useEffect } from "react";
import { useSignupContext } from "../context/SignupContext";
import { CryptoService } from "../services/crypto";
import AgreeToContinue from "./AgreeToContinue";

export default function MnemonicDisplay() {
  const { mnemonic, setMnemonic } = useSignupContext();

  useEffect(() => {
    const generatedMnemonic = CryptoService.generateMnemonic();
    setMnemonic(generatedMnemonic);
  }, [setMnemonic]);

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
          {mnemonic.map((word, index) => (
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
            <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
              <path d="M8 3a1 1 0 011-1h2a1 1 0 110 2H9a1 1 0 01-1-1z" />
              <path d="M6 3a2 2 0 00-2 2v11a2 2 0 002 2h8a2 2 0 002-2V5a2 2 0 00-2-2 3 3 0 01-3 3H9a3 3 0 01-3-3z" />
            </svg>
            Copy
          </button>
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
      <AgreeToContinue
        id="seedPhraseConfirmation"
        label="I have written down my seed phrase on paper and stored it securely"
      />
    </div>
  );
}
