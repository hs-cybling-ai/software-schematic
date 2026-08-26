import { defineConfig } from 'vite';
import { resolve } from 'node:path';

export default defineConfig({
  base: './',
  build: {
    outDir: resolve(import.meta.dirname, '../software-schematic-cli/assets/web'),
    emptyOutDir: true,
    assetsDir: 'vendor',
    sourcemap: false,
  },
});
