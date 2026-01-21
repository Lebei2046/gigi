import { useEffect } from 'react'
import { loadAuthData } from '../store/authSlice'
import { useAppDispatch, useAppSelector } from '../store'

export default function useInitAuth() {
  const dispatch = useAppDispatch()
  const { status } = useAppSelector(state => state.auth)

  // Only run once on mount to check authentication status
  // Empty dependency array ensures this doesn't re-run on every render
  useEffect(() => {
    dispatch(loadAuthData())
  }, []) // Empty deps = run once on mount

  return {
    status,
  }
}
