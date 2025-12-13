import { Button } from '@/components/ui/button'
import {
  Card,
  CardAction,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { useSignupContext } from '../context/SignupContext'

export default function Welcome() {
  const { dispatch } = useSignupContext()

  return (
    <div className="min-h-screen flex flex-col items-center justify-center bg-gray-50 p-4">
      <div className="w-full max-w-2xl space-y-6">
        <div className="text-center">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">
            Let's set up your wallet account
          </h1>
          <p className="text-gray-600">Pick an option below to get started</p>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>Create new wallet</CardTitle>
            <CardDescription>
              Create a fresh wallet and generate a new seed phrase
            </CardDescription>
            <CardAction>
              <Button
                variant="link"
                onClick={() =>
                  dispatch({ type: 'INIT_SIGNUP', payload: 'create' })
                }
              >
                Create
              </Button>
            </CardAction>
          </CardHeader>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Import seed phrase</CardTitle>
            <CardDescription>
              Restore an existing wallet using your seed phrase
            </CardDescription>
            <CardAction>
              <Button
                variant="link"
                onClick={() =>
                  dispatch({ type: 'INIT_SIGNUP', payload: 'import' })
                }
              >
                Import
              </Button>
            </CardAction>
          </CardHeader>
        </Card>
      </div>
    </div>
  )
}
