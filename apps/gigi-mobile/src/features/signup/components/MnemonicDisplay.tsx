import { useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { generateMnemonics } from '@/utils/crypto'
import AgreeToContinue from './AgreeToContinue'
import { useSignupContext } from '../context/SignupContext'

export default function MnemonicDisplay() {
  const STEP: number = 1

  const {
    state: { mnemonic },
    dispatch,
  } = useSignupContext()

  useEffect(() => {
    const generatedMnemonic = generateMnemonics()
    dispatch({ type: 'SET_MNEMONIC', payload: generatedMnemonic })
  }, [dispatch])

  const handleCopy = () => {
    const mnemonicString = mnemonic.join(' ')
    navigator.clipboard
      .writeText(mnemonicString)
      .then(() => {
        // Show a more subtle notification instead of alert
        const notification = document.createElement('div')
        notification.textContent = 'Copied to clipboard!'
        notification.className =
          'fixed top-4 right-4 bg-green-500 text-white px-4 py-2 rounded-lg shadow-lg z-50 transition-opacity duration-300'
        document.body.appendChild(notification)

        setTimeout(() => {
          notification.style.opacity = '0'
          setTimeout(() => document.body.removeChild(notification), 300)
        }, 2000)
      })
      .catch(err => {
        console.error('Failed to copy:', err)
      })
  }

  return (
    <div className="space-y-6">
      <div className="text-center space-y-2">
        <h1 className="text-2xl font-bold text-gray-900">Seed Phrase</h1>
        <p className="text-gray-600 px-4">
          Please write down your seed phrase in the correct order and keep it in
          a safe place.
        </p>
      </div>

      <div className="bg-white rounded-xl border border-gray-200 p-4 shadow-sm">
        <div className="grid grid-cols-3 gap-3">
          {mnemonic.map((word: string, index: number) => (
            <div
              key={index}
              className="flex items-center space-x-2 bg-gray-50 rounded-lg px-3 py-2 border border-gray-200"
            >
              <span className="text-sm font-medium text-gray-500 min-w-[20px]">
                {index + 1}.
              </span>
              <span className="text-sm font-medium text-gray-900">{word}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="flex justify-center">
        <Button
          onClick={handleCopy}
          className="w-full max-w-xs bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 rounded-xl transition-colors duration-200"
        >
          Copy Seed Phrase
        </Button>
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
          label="I have written down my seed phrase on paper and stored it securely"
          step={STEP}
        />
      </div>
    </div>
  )
}
