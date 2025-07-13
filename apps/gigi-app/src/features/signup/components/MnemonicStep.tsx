import { useAppDispatch } from '@/store';
import { nextStep, setMnemonic } from '../store/signupSlice';
import { CryptoService } from '../services/crypto';

export function MnemonicStep() {
  const dispatch = useAppDispatch();
  const mnemonicArray = CryptoService.generateMnemonic();
  const mnemonic = mnemonicArray.join(' ');

  return (
    <div>
      <h2>Your Mnemonic Phrase</h2>
      <div>{mnemonic}</div>
      <button onClick={() => {
        dispatch(setMnemonic(mnemonic));
        dispatch(nextStep());
      }}>Continue</button>
    </div>
  );
}
