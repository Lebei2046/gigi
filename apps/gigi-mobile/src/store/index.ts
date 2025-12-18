import { configureStore } from '@reduxjs/toolkit'
import { useDispatch, useSelector } from 'react-redux'
import type { TypedUseSelectorHook } from 'react-redux'
import authReducer from './authSlice'
import logsReducer from './logsSlice'
import chatReducer from './chatSlice'

export const store = configureStore({
  reducer: {
    auth: authReducer,
    logs: logsReducer,
    chat: chatReducer,
  },
  middleware: getDefaultMiddleware =>
    getDefaultMiddleware({
      serializableCheck: {
        // Ignore these action types if any non-serializable data still appears
        ignoredActions: [],
        // Ignore these paths in the state if needed
        ignoredPaths: [],
      },
    }),
})

export type RootState = ReturnType<typeof store.getState>
export type AppDispatch = typeof store.dispatch
export const useAppDispatch = () => useDispatch<AppDispatch>()
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector
