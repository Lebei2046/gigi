import { useSignupContext } from "../context/SignupContext";

export default function Welcome() {
  const { initSignup } = useSignupContext();

  return (
    <div className="flex flex-col items-center justify-center min-h-screen bg-gray-100 p-8">
      <div className="mb-12">
        <h1 className="text-3xl font-bold text-gray-800 mb-4 text-left">Let's set up your wallet account</h1>
        <p className="text-lg text-gray-600 text-left">Pick an option below to get started</p>
      </div>
      <div className="flex flex-col md:flex-row gap-8 w-full max-w-4xl">
        <div
          className="card w-96 bg-base-100 shadow-xl hover:shadow-2xl transition-shadow cursor-pointer"
          onClick={() => initSignup("create")}
        >
          <figure className="px-10 pt-10">
            <img src="https://picsum.photos/200/300?random=1" alt="pic" className="w-full h-24 object-cover" />
          </figure>
          <div className="card-body text-left">
            <h2 className="card-title">Create new wallet</h2>
            <p>Create a fresh wallet and generate a new seed phrase</p>
          </div>
        </div>
        <div
          className="card w-96 bg-base-100 shadow-xl hover:shadow-2xl transition-shadow cursor-pointer"
          onClick={() => initSignup("import")}
        >
          <figure className="px-10 pt-10">
            <img src="https://picsum.photos/200/300?random=2" alt="pic" className="w-full h-24 object-cover" />
          </figure>
          <div className="card-body">
            <h2 className="card-title">Import seed phrase</h2>
            <p className="text-left">Restore an existing wallet using your seed phrase</p>
          </div>
        </div>
      </div>
    </div>
  );
}
