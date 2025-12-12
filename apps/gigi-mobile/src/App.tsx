import { BrowserRouter, Routes, Route } from "react-router-dom";
import React from "react";
import useInitAuth from "./hooks/useInitAuth.ts";
import { unregisteredRoutes, unauthenticatedRoutes, authenticatedRoutes } from "./routes.tsx";
import { clearIndexedDB } from "@/utils/storage";

// Remove this line in production
const DEBUG_MODE = import.meta.env.DEV;

export default function App() {
  const { status } = useInitAuth();

  const routes = status === 'unregistered'
    ? unregisteredRoutes
    : status === 'unauthenticated'
      ? unauthenticatedRoutes
      : authenticatedRoutes;

  // Debug keyboard shortcut (Ctrl+Shift+D)
  React.useEffect(() => {
    if (!DEBUG_MODE) return;

    const handleKeyPress = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.shiftKey && e.key === 'D') {
        e.preventDefault();
        clearIndexedDB().then(() => {
          alert('IndexedDB cleared! Reloading...');
          window.location.reload();
        });
      }
    };

    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, []);

  return (
    <div className="min-h-screen pt-[env(safe-area-inset-top)] pb-[env(safe-area-inset-bottom)]">
      <BrowserRouter>
        <Routes>
          {routes.map(({ path, element, caseSensitive }) => (
            <Route
              key={path}
              path={path}
              element={element}
              caseSensitive={caseSensitive}
            />
          ))}
        </Routes>
      </BrowserRouter>
    </div>
  );
}