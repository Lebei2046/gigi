import { useEffect } from "react";
import { useSelector, useDispatch } from "react-redux";
import type { RootState } from "./store";
import { initAuth } from "./store/authSlice";
import Signup from "./features/signup/Signup";
import Unlock from "./features/login/Unlock";
import Home from "./features/home/Home";

export default function App() {
  const dispatch = useDispatch();
  useEffect(() => {
    dispatch(initAuth());
  }, [dispatch]);
  const { status } = useSelector((state: RootState) => state.auth);

  return (
    <div className="min-h-screen bg-base-100 p-8">
      {status === 'unregistered' && <Signup />}
      {status === 'unauthenticated' && <Unlock />}
      {status === 'authenticated' && <Home />}
    </div>
  );
}
