import { useEffect, useState } from "react";
import { useSignupContext } from "../context/SignupContext";
import { encryptMnemonics, generateAddress } from "../../../utils/crypto";
import { setStorageItem } from "../../../utils/storage";

const SignupFinish = () => {
  const { mnemonic, password } = useSignupContext();
  const [address, setAddress] = useState("");

  useEffect(() => {
    // 生成钱包地址
    const walletAddress = generateAddress(mnemonic);
    setAddress(walletAddress);

    // 加密助记词并保存
    const { mnemonic: cryptedMnemonic, nonce } = encryptMnemonics(mnemonic, password);
    setStorageItem("gigi", {
      nonce,
      mnemonic: cryptedMnemonic,
      address: walletAddress,
    });
  }, [mnemonic, password]);

  return (
    <div className="min-h-screen bg-base-100 p-8">
      <h1 className="text-2xl font-bold mb-4">Your account created successfully</h1>
      <p className="text-gray-600 mb-8">Below is your first Gigi wallet account</p>

      <div className="bg-white p-6 rounded-lg shadow-md mb-8">
        <h2 className="text-lg font-semibold mb-2">Account Details</h2>
        <p className="text-gray-700">Wallet Address: {address}</p>
      </div>

      <button className="btn btn-primary w-full">Go to login</button>
    </div>
  );
};

export default SignupFinish;
