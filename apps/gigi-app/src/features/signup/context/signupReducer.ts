import type { Reducer } from "react";
import { encryptMnemonics, generateAddress } from "../../../utils/crypto";
import { setStorageItem } from "../../../utils/storage";

export type SignupType = "create" | "import" | null;

type StepType = { index: number; checked: boolean };

type SignupState = {
  currentStep: number;
  steps: boolean[];
  signupType: SignupType;
  mnemonic: string[];
  password: string;
  address: string;
  name: string;
};

export type SignupAction =
  | { type: "GO_TO_NEXT_STEP" }
  | { type: "GO_TO_PREV_STEP" }
  | { type: "SAVE_ACCOUNT_INFO" }
  | { type: "SET_MNEMONIC"; payload: string[] }
  | { type: "SET_PASSWORD"; payload: string }
  | { type: "SET_NAME"; payload: string }
  | { type: "SET_STEP_CHECKED"; payload: StepType }
  | { type: "INIT_SIGNUP"; payload: SignupType };

export const initialState: SignupState = {
  currentStep: 0,
  steps: Array(4).fill(false),
  signupType: null,
  mnemonic: Array(12).fill(""),
  password: "",
  address: "",
  name: "",
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
      };
    case "GO_TO_PREV_STEP":
      return {
        ...state,
        currentStep: Math.max(0, state.currentStep - 1),
        signupType: state.currentStep === 0 ? null : state.signupType,
      };
    case "SET_MNEMONIC":
      return { ...state, mnemonic: action.payload };
    case "SET_PASSWORD":
      return { ...state, password: action.payload };
    case "SET_NAME":
      return { ...state, name: action.payload };
    case "INIT_SIGNUP":
      return {
        ...state,
        signupType: action.payload,
        steps: Array(4).fill(false),
        mnemonic: Array(12).fill(""),
        password: "",
      };
    case "SET_STEP_CHECKED":
      return {
        ...state,
        steps: state.steps.map((step, index) =>
          index === action.payload.index ? action.payload.checked : step
        ),
      };
    case "SAVE_ACCOUNT_INFO":
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
          name: state.name,
        });
        return {
          ...state,
          address: walletAddress,
        };
      }
    default:
      return state;
  }
};