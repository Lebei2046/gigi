import { useEffect, useState } from 'react'
import { Input } from '@/components/ui/input'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import AgreeToContinue from './AgreeToContinue'
import { useSignupContext } from '../context/SignupContext'

export default function MnemonicInput() {
  const STEP: number = 1

  const {
    state: { mnemonic },
    dispatch,
  } = useSignupContext()
  const [isCheckboxDisabled, setIsCheckboxDisabled] = useState<boolean>(true)

  const handleChange = (index: number, value: string) => {
    const newMnemonic = [...mnemonic]
    newMnemonic[index] = value
    dispatch({ type: 'SET_MNEMONIC', payload: newMnemonic })
  }

  useEffect(() => {
    const isAllFilled = mnemonic.every(word => word.trim() !== '')
    setIsCheckboxDisabled(!isAllFilled)
  }, [mnemonic])

  return (
    <div>
      <h1>Recover Account</h1>
      <p>Enter your existing seed phrase to restore your wallet.</p>
      <div>
        {Array.from({ length: 12 }).map((_, index) => (
          <div key={index}>
            <span>{index + 1}.</span>
            <Input
              type="text"
              placeholder="word"
              value={mnemonic[index]}
              onChange={event => handleChange(index, event.target.value)}
            />
          </div>
        ))}
      </div>
      <Alert variant="default">
        <AlertTitle>Heads up!</AlertTitle>
        <AlertDescription>
          Anyone with access to your recovery phrase can access your assets.
          Store it securely. Gigi does not keep a backup of your 12-word phrase.
        </AlertDescription>
      </Alert>
      <AgreeToContinue
        id="seedPhraseConfirmation"
        label="I have written down my seed phrase on paper and stored it securely"
        step={STEP}
        disabled={isCheckboxDisabled}
      />
    </div>
  )
}
