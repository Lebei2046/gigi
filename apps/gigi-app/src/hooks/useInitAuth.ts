import { useEffect } from 'react';
import { initAuth } from '../store/authSlice';
import { useAppDispatch, useAppSelector } from "../store";

export default function useInitAuth() {
  const dispatch = useAppDispatch();
  const { status } = useAppSelector((state) => state.auth);

  useEffect(() => {
    dispatch(initAuth());
  }, [dispatch]);

  return {
    status,
  }
}
