const SignupFinish = () => {
  return (
    <div className="min-h-screen bg-base-100 p-8">
      <h1 className="text-2xl font-bold mb-4">Your account created successfully</h1>
      <p className="text-gray-600 mb-8">Below is your first Gigi wallet account</p>

      <div className="bg-white p-6 rounded-lg shadow-md mb-8">
        <h2 className="text-lg font-semibold mb-2">Account Details</h2>
        <p className="text-gray-700">Username: user123</p>
        <p className="text-gray-700">Email: user123@example.com</p>
      </div>

      <button className="btn btn-primary w-full">Go to login</button>
    </div>
  );
};

export default SignupFinish;
