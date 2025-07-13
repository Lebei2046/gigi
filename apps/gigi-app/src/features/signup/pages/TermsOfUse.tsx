import { ReactComponent as Terms } from '/public/TermsOfUse.md';
export default function TermsOfUse() {
  return (
    <div className="p-8">
      <h1 className="text-2xl font-bold mb-4">Terms of Use Agreement</h1>
      <p className="mb-4 text-gray-600">Please read the following terms and conditions carefully, check the box to agree and continue.</p>
      <div className="overflow-y-auto overflow-x-hidden max-h-92 border rounded-lg p-4">
        <Terms />
      </div>
      <div className="mt-4">
        <label className="flex items-center space-x-2">
          <input type="checkbox" className="checkbox" />
          <span>I agree to the Terms of Use Agreement</span>
        </label>
      </div>
      <div className="mt-6 flex justify-between">
        <button className="btn btn-outline w-40">Back</button>
        <button className="btn btn-primary w-40">Next: Seed Phrase</button>
      </div>
    </div>
  );
}
