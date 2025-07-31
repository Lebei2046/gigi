import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],

	// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
	// prevent Vite from obscuring rust errors
	clearScreen: false,
	// tauri expects a fixed port, fail if that port is not available
	server: {
		host: host || false,
		port: 1420,
		strictPort: true,
		hmr: host ? {
			protocol: 'ws',
			host,
			port: 1421
		} : undefined,
	},
});
