import type { Reducer } from "react";
import { encryptMnemonics, generateAddress } from "../../../utils/crypto";
import { setStorageItem } from "../../../utils/storage";

type SignupType = "create" | "import" | null;

type SignupState = {
  currentStep: number;
  isNextDisabled: boolean;
  signupType: SignupType;
  mnemonic: string[];
  password: string;
  address: string;
};

type SignupAction =
  | { type: "GO_TO_NEXT_STEP" }
  | { type: "GO_TO_PREV_STEP" }
  | { type: "GEN_ADDRESS_AND_ENC_SAVE_MNEMONIC" }
  | { type: "SET_IS_NEXT_DISABLED"; payload: boolean }
  | { type: "SET_SIGNUP_TYPE"; payload: SignupType }
  | { type: "SET_MNEMONIC"; payload: string[] }
  | { type: "SET_PASSWORD"; payload: string }
  | { type: "INIT_SIGNUP"; payload: SignupType };

export const initialState: SignupState = {
  currentStep: 0,
  isNextDisabled: true,
  signupType: null,
  mnemonic: Array(12).fill(""),
  password: "",
  address: "",
};

export const signupReducer: Reducer<SignupState, SignupAction> = (
  state,
  action
) => {
  switch (action.type) {
    case "GO_TO_NEXT_STEP":
      return {
        ...state,
        currentStep: state.currentStep + 1,
        isNextDisabled: true,
      };
    case "GO_TO_PREV_STEP":
      return {
        ...state,
        currentStep: Math.max(0, state.currentStep - 1),
        signupType: state.currentStep === 0 ? null : state.signupType,
      };
    case "SET_IS_NEXT_DISABLED":
      return { ...state, isNextDisabled: action.payload };
    case "SET_SIGNUP_TYPE":
      return { ...state, signupType: action.payload };
    case "SET_MNEMONIC":
      return { ...state, mnemonic: action.payload };
    case "SET_PASSWORD":
      return { ...state, password: action.payload };
    case "INIT_SIGNUP":
      return {
        ...state,
        signupType: action.payload,
        isNextDisabled: true,
        mnemonic: Array(12).fill(""),
      };
    case "GEN_ADDRESS_AND_ENC_SAVE_MNEMONIC":
      {
        const walletAddress = generateAddress(state.mnemonic);
        const { mnemonic: cryptedMnemonic, nonce } = encryptMnemonics(
          state.mnemonic,
          state.password
        );
        setStorageItem("gigi", {
          nonce,
          mnemonic: cryptedMnemonic,
          address: walletAddress,
        });
        return {
          ...state,
          address: walletAddress,
          isNextDisabled: false,
        };
      }
    default:
      return state;
  }
};