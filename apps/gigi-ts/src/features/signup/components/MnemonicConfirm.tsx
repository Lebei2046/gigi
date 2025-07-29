import { useState, useEffect } from "react";
import { useSignupContext } from "../context/SignupContext";

export default function MnemonicConfirm() {
  const STEP: number = 2;

  const { state: { mnemonic }, dispatch } = useSignupContext();
  const [randomIndices, setRandomIndices] = useState<number[]>([]);
  const [userInputs, setUserInputs] = useState<Record<number, string>>({});

  // Initialize random indices on first render
  useEffect(() => {
    const indices = Array.from({ length: 12 }, (_, i) => i);
    const shuffled = [...indices].sort(() => 0.5 - Math.random());
    setRandomIndices(shuffled.slice(0, 3));
  }, []);

  useEffect(() => {
    const allCorrect = randomIndices.every(idx => {
      return userInputs[idx]?.toLowerCase() === mnemonic[idx]?.toLowerCase();
    });
    dispatch({ type: "SET_STEP_CHECKED", payload: { index: STEP, checked: allCorrect } });
  }, [userInputs, randomIndices, mnemonic, dispatch]);

  const handleInputChange = (index: number, value: string) => {
    const updatedInputs = { ...userInputs, [index]: value };
    setUserInputs(updatedInputs);
  };

  const isInputCorrect = (index: number) => {
    return userInputs[index]?.toLowerCase() === mnemonic[index]?.toLowerCase();
  };

  return (
    <div className="p-8 bg-white">
      <div>
        <h1 className="text-2xl font-bold mb-4">Confirm phrase</h1>
        <p className="mb-4 text-gray-600">
          Enter the missing words to confirm your seed phrase.
        </p>
      </div>
      <div className="mb-6">
        <div className="grid grid-cols-3 gap-2">
          {mnemonic.map((word, index) => {
            const isInput = randomIndices.includes(index);
            return (
              <div key={index} className="p-2 text-left">
                <span className="text-gray-500 text-right w-8 inline-block">{index + 1}.</span>
                {isInput ? (
                  <input
                    type="text"
                    className={`ml-2 px-2 py-1 w-24 border-b ${isInputCorrect(index) ? 'border-green-500' : 'border-gray-300'}`}
                    placeholder="Enter word"
                    value={userInputs[index] || ""}
                    onChange={(e) => handleInputChange(index, e.target.value)}
                  />
                ) : (
                  <span className="ml-2 font-medium border-b border-gray-300">{word}</span>
                )}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
