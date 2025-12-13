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
        alert('Copied to clipboard!')
      })
      .catch(err => {
        console.error('Failed to copy:', err)
      })
  }

  return (
    <div>
      <h1>Seed Phrase</h1>
      <p>
        Please write down your seed phrase in the correct order and keep it in a
        safe place.
      </p>
      <div>
        {mnemonic.map((word: string, index: number) => (
          <div key={index}>
            <span>{index + 1}.</span>
            <span>{word}</span>
          </div>
        ))}
      </div>
      <Button onClick={handleCopy}>Copy</Button>
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
      />
    </div>
  )
}
