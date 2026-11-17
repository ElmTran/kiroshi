import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

// https://v2.tauri.app/start/frontend/vite/
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    host: '127.0.0.1',
    // 1420 is taken by mikoshi's dev server.
    port: 1430,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/target/**', '**/src-tauri/src/**'] },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_'],
  build: {
    // WebView2 on Windows 10/11 is evergreen Chromium
    target: 'chrome120',
    sourcemap: Boolean(process.env.TAURI_ENV_DEBUG),
  },
  test: {
    include: ['src/**/*.test.ts'],
  },
});
