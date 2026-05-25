import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import packageJson from './package.json' with { type: 'json' };

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  define: {
    __APP_VERSION__: JSON.stringify(packageJson.version)
  },
  server: {
    host: '127.0.0.1',
    port: 1420,
    strictPort: true
  },
  envPrefix: ['VITE_', 'TAURI_']
});
