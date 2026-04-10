import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['tests/**/*.{test,spec}.{ts,tsx}'],
    globals: true,
    environment: 'node',
    timeout: 60000, // Increase timeout for integration tests
    run: {
      serial: true, // Run tests sequentially to avoid port conflicts
    },
    silent: true, // Hide console logs
  },
});
