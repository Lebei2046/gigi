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
    <div>
      <h1>Confirm phrase</h1>
      <p>Enter the missing words to confirm your seed phrase.</p>
      <div>
        {mnemonic.map((word, index) => {
          const isInput = randomIndices.includes(index)
          return (
            <div key={index}>
              <span>{index + 1}.</span>
              {isInput ? (
                <Input
                  type="text"
                  placeholder="Enter word"
                  value={userInputs[index] || ''}
                  onChange={e => handleInputChange(index, e.target.value)}
                />
              ) : (
                <span>{word}</span>
              )}
            </div>
          )
        })}
      </div>
    </div>
  )
}
