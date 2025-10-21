import { BrowserRouter, Routes, Route } from "react-router-dom";
import useInitAuth from "./hooks/useInitAuth.ts";
import { unregisteredRoutes, unauthenticatedRoutes, authenticatedRoutes } from "./routes.tsx";

export default function App() {
  const { status } = useInitAuth();

  const routes = status === 'unregistered'
    ? unregisteredRoutes
    : status === 'unauthenticated'
      ? unauthenticatedRoutes
      : authenticatedRoutes;

  return (
    <BrowserRouter>
      <Routes>
        {routes.map((route) => (
          <Route key={route.path} {...route} />
        ))}
      </Routes>
    </BrowserRouter>
  );
}
