import { useDispatch, useSelector } from 'react-redux';
import type { RootState } from '../../store';
import { increment, decrement } from './counterSlice';

export default function Counter() {
  const count = useSelector((state: RootState) => state.counter.value);
  const dispatch = useDispatch();

  return (
    <div className="flex flex-col items-center gap-4 p-4">
      <h1 className="text-2xl font-bold">Counter</h1>
      <div className="flex items-center gap-4">
        <button 
          className="btn btn-primary" 
          onClick={() => dispatch(decrement())}
        >
          -
        </button>
        <span className="text-xl">{count}</span>
        <button 
          className="btn btn-primary" 
          onClick={() => dispatch(increment())}
        >
          +
        </button>
      </div>
    </div>
  );
}