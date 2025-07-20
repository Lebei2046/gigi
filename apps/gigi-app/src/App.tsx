import { useEffect, type JSX } from "react";
import { useSelector, useDispatch } from "react-redux";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import type { RootState } from "./store";
import { initAuth } from "./store/authSlice";
import Signup from "./features/signup/Signup";
import Unlock from "./features/login/Unlock";
import ResetAccount from "./features/login/ResetAccount";
import Home from "./features/home/Home";
import ChatPage from "./features/chat/Chat";

const ProtectedRoute = ({ children, requiredStatus }: { children: JSX.Element; requiredStatus: string[] }) => {
  const { status } = useSelector((state: RootState) => state.auth);

  if (!requiredStatus.includes(status)) {
    const url = status === 'unregistered' ? '/signup' : status === 'unauthenticated' ? '/login' : '/';
    return <Navigate to={url} replace />;
  }

  return children;
}

export default function App() {
  const dispatch = useDispatch();
  useEffect(() => {
    dispatch(initAuth());
  }, [dispatch]);

  return (
    <div className="min-h-screen bg-base-100 p-8">
      <BrowserRouter>
        <Routes>
          <Route path="/signup" element={
            <ProtectedRoute requiredStatus={['unregistered']}>
              <Signup />
            </ProtectedRoute>
          } />
          <Route path="/reset" element={
            <ResetAccount />
          } />
          <Route path="/login" element={
            <ProtectedRoute requiredStatus={['unauthenticated']}>
              <Unlock />
            </ProtectedRoute>
          } />
          <Route path="/" element={
            <ProtectedRoute requiredStatus={['authenticated']}>
              <Home />
            </ProtectedRoute>
          } />
          <Route path="/chat/:id" element={
            <ProtectedRoute requiredStatus={['authenticated']}>
              <ChatPage />
            </ProtectedRoute>
          } />
        </Routes>
      </BrowserRouter>
    </div>
  );
}
