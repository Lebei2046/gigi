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
  );
}
