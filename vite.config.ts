import { defineConfig } from 'vite';
import { resolve } from 'node:path';

const root = import.meta.dirname;

export default defineConfig({
  clearScreen: false,
  envPrefix: ['VITE_', 'TAURI_'],
  server: { port: 1420, strictPort: true },
  build: {
    target: 'safari15',
    rolldownOptions: {
      input: {
        duck: resolve(root, 'index.html'),
        settings: resolve(root, 'settings.html'),
        fx: resolve(root, 'fx.html'),
      },
    },
  },
  test: { include: ['src/**/*.test.ts'] },
});
