import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { Provider } from "react-redux";

import { store } from "./store";
import App from './App'
import './index.css'

// Standard rendering
createRoot(document.getElementById('root')!).render(
  <StrictMode>
    {/* Provide the store to the app */}
    <Provider store={store}>
      <App />
    </Provider>
  </StrictMode>,
)