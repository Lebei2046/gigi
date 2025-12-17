import { ScrollArea } from '@/components/ui/scroll-area'
import { ReactComponent as Terms } from '@/assets/TermsOfUse.md'
import AgreeToContinue from './AgreeToContinue'

export default function TermsOfUse() {
  const STEP: number = 0

  return (
    <div className="space-y-6">
      <div className="text-center space-y-2">
        <h1 className="text-2xl font-bold text-gray-900">Terms of Use</h1>
        <p className="text-gray-600 px-4">
          Please read the following terms and conditions carefully
        </p>
      </div>

      <div className="bg-white rounded-xl border border-gray-200 shadow-sm">
        <div className="p-4 max-h-64 overflow-y-auto">
          <ScrollArea className="h-full">
            <div className="text-sm text-gray-700 leading-relaxed">
              <Terms />
            </div>
          </ScrollArea>
        </div>
      </div>

      <AgreeToContinue
        id="termsOfUseAgreement"
        label="I agree to the Terms of Use Agreement"
        step={STEP}
      />
    </div>
  )
}
