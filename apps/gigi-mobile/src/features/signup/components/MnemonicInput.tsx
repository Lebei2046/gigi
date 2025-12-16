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
    <div className="space-y-6">
      <div className="text-center space-y-2">
        <h1 className="text-2xl font-bold text-gray-900">Recover Account</h1>
        <p className="text-gray-600 px-4">
          Enter your existing seed phrase to restore your account.
        </p>
      </div>

      <div className="bg-white rounded-xl border border-gray-200 p-4 shadow-sm">
        <div className="grid grid-cols-2 gap-3">
          {Array.from({ length: 12 }).map((_, index) => (
            <div key={index} className="flex items-center space-x-2">
              <span className="text-sm font-medium text-gray-500 min-w-[20px]">
                {index + 1}.
              </span>
              <Input
                type="text"
                placeholder="word"
                value={mnemonic[index]}
                onChange={event => handleChange(index, event.target.value)}
                className="flex-1 text-sm"
              />
            </div>
          ))}
        </div>
      </div>

      <Alert variant="default" className="bg-amber-50 border-amber-200">
        <AlertTitle className="text-amber-800 font-semibold">
          ⚠️ Important
        </AlertTitle>
        <AlertDescription className="text-amber-700">
          Anyone with access to your recovery phrase can access your account.
          Store it securely. Gigi does not keep a backup of your 12-word phrase.
        </AlertDescription>
      </Alert>

      <div className="pt-4">
        <AgreeToContinue
          id="seedPhraseConfirmation"
          label="I have entered my seed phrase correctly"
          step={STEP}
          disabled={isCheckboxDisabled}
        />
      </div>
    </div>
  )
}
