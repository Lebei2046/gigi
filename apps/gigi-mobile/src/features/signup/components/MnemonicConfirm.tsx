import { useState, useEffect } from 'react'
import { Input } from '@/components/ui/input'
import { useSignupContext } from '../context/SignupContext'

export default function MnemonicConfirm() {
  const STEP: number = 2

  const {
    state: { mnemonic },
    dispatch,
  } = useSignupContext()
  const [randomIndices, setRandomIndices] = useState<number[]>([])
  const [userInputs, setUserInputs] = useState<Record<number, string>>({})

  // Initialize random indices on first render
  useEffect(() => {
    const indices = Array.from({ length: 12 }, (_, i) => i)
    const shuffled = [...indices].sort(() => 0.5 - Math.random())
    setRandomIndices(shuffled.slice(0, 3))
  }, [])

  useEffect(() => {
    const allCorrect = randomIndices.every(idx => {
      return userInputs[idx]?.toLowerCase() === mnemonic[idx]?.toLowerCase()
    })
    dispatch({
      type: 'SET_STEP_CHECKED',
      payload: { index: STEP, checked: allCorrect },
    })
  }, [userInputs, randomIndices, mnemonic, dispatch])

  const handleInputChange = (index: number, value: string) => {
    const updatedInputs = { ...userInputs, [index]: value }
    setUserInputs(updatedInputs)
  }

  const isInputCorrect = (index: number) => {
    return userInputs[index]?.toLowerCase() === mnemonic[index]?.toLowerCase()
  }

  return (
    <div className="space-y-6">
      <div className="text-center space-y-2">
        <h1 className="text-2xl font-bold text-gray-900">Confirm Phrase</h1>
        <p className="text-gray-600 px-4">
          Enter the missing words to confirm you've saved your seed phrase
          correctly.
        </p>
      </div>

      <div className="bg-white rounded-xl border border-gray-200 p-4 shadow-sm">
        <div className="grid grid-cols-3 gap-3">
          {mnemonic.map((word, index) => {
            const isInput = randomIndices.includes(index)
            const isCorrect =
              isInput && userInputs[index] ? isInputCorrect(index) : null

            return (
              <div
                key={index}
                className={`
                  flex items-center space-x-2 rounded-lg px-3 py-2 border transition-colors duration-200
                  ${
                    isInput
                      ? isCorrect === true
                        ? 'bg-green-50 border-green-300'
                        : isCorrect === false
                          ? 'bg-red-50 border-red-300'
                          : 'bg-white border-gray-300'
                      : 'bg-gray-50 border-gray-200'
                  }
                `}
              >
                <span className="text-sm font-medium text-gray-500 min-w-[20px]">
                  {index + 1}.
                </span>
                {isInput ? (
                  <Input
                    type="text"
                    placeholder="Enter word"
                    value={userInputs[index] || ''}
                    onChange={e => handleInputChange(index, e.target.value)}
                    className="flex-1 text-sm border-0 bg-transparent p-0 focus-visible:ring-0"
                  />
                ) : (
                  <span className="text-sm font-medium text-gray-900">
                    {word}
                  </span>
                )}
              </div>
            )
          })}
        </div>
      </div>

      <div className="text-center text-sm text-gray-600">
        <p>Complete all fields correctly to continue</p>
      </div>
    </div>
  )
}
