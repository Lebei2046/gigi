import { lazy } from 'react';
import type { RouteObject } from 'react-router-dom';

const Signup = lazy(() => import('./features/signup/Signup'));
const Unlock = lazy(() => import('./features/login/Unlock'));
const ResetAccount = lazy(() => import('./features/login/ResetAccount'));
const Home = lazy(() => import('./features/home/Home'));
const Chat = lazy(() => import('./features/chat/Chat'));
const Me = lazy(() => import('./features/me/Me'));

export const unregisteredRoutes: RouteObject[] = [
  {
    path: '*',
    element: <Signup />,
  },
];

export const unauthenticatedRoutes: RouteObject[] = [
  {
    path: '/reset',
    element: <ResetAccount />,
  },
  {
    path: '*',
    element: <Unlock />,
  },
];

export const authenticatedRoutes: RouteObject[] = [
  {
    path: '/',
    element: <Home />,
  },
  {
    path: '/chat/:id',
    element: <Chat />,
  },
  {
    path: '/me',
    element: <Me />,
  },
  {
    path: '*',
    element: <Home />,
  },
];
