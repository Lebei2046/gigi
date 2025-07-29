import { ReactComponent as Terms } from '@/assets/TermsOfUse.md';
import AgreeToContinue from './AgreeToContinue';

export default function TermsOfUse() {
  const STEP: number = 0;

  return (
    <div className="p-8">
      <h1 className="text-2xl font-bold mb-4">Terms of Use Agreement</h1>
      <p className="mb-4 text-gray-600">Please read the following terms and conditions carefully, check the box to agree and continue.</p>
      <div className="overflow-y-auto overflow-x-hidden max-h-92 border rounded-lg p-4">
        <Terms />
      </div>
      <AgreeToContinue
        id="termsOfUseAgreement"
        label="I agree to the Terms of Use Agreement"
        step={STEP}
      />
    </div>
  );
}
