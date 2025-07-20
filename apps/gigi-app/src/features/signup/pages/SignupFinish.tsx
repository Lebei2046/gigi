import { useEffect } from "react";
import { useDispatch } from "react-redux";
import { useNavigate } from "react-router-dom";
import { initAuth } from "../../../store/authSlice";
import { useSignupContext } from "../context/SignupContext";

export default function SignupFinish() {
  const navigate = useNavigate();
  const dispatch = useDispatch();
  const { state: { address, name }, dispatch: signupDispatch } = useSignupContext();

  useEffect(() => {
    signupDispatch({ type: "SAVE_ACCOUNT_INFO" });
  }, [signupDispatch]);

  return (
    <div className="min-h-screen bg-base-100 p-8">
      <h1 className="text-2xl font-bold mb-4">Your account created successfully</h1>
      <p className="text-gray-600 mb-8">Below is your first Gigi wallet account</p>

      <div className="bg-white p-6 rounded-lg shadow-md mb-8">
        <h2 className="text-lg font-semibold mb-2">Account Details</h2>
        <p className="text-gray-700">Account Name: {name}</p>
        <p className="text-gray-700">Wallet Address: {address}</p>
      </div>

      <button className="btn btn-primary w-full" onClick={() => {
        dispatch(initAuth());
        navigate('/login');
      }}>Go to login</button>
    </div>
  );
}
