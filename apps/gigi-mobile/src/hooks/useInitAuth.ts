import { useEffect } from 'react';
import { loadAuthData } from '../store/authSlice';
import { useAppDispatch, useAppSelector } from "../store";

export default function useInitAuth() {
  const dispatch = useAppDispatch();
  const { status } = useAppSelector((state) => state.auth);

  useEffect(() => {
    const initAuth = async () => {
      dispatch(loadAuthData());
    };

    initAuth();
  }, [dispatch]);

  return {
    status,
  }
}
