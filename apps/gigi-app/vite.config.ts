import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import uocss from 'unocss/vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [uocss(), react()],
})
