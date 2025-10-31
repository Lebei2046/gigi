import { lazy } from 'react';
import type { RouteObject } from 'react-router-dom';

const Signup = lazy(() => import('./features/signup/Signup'));
const Unlock = lazy(() => import('./features/signin/Unlock'));
const ResetAccount = lazy(() => import('./features/signin/ResetAccount'));
const Home = lazy(() => import('./features/home/Home'));
// const Chat = lazy(() => import('./features/chat/Chat'));

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
  // {
  //   path: '/chat/:id',
  //   element: <Chat />,
  // },
  {
    path: '*',
    element: <Home />,
  },
];
