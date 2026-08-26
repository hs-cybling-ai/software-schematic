import { defineConfig } from 'vite';
import { resolve } from 'node:path';

export default defineConfig({
  base: './',
  plugins: [{
    name: 'wkwebview-file-url-compatibility',
    enforce: 'post',
    transformIndexHtml(html) {
      return html
        .replace(/<script type="module" crossorigin/g, '<script defer')
        .replace(/ crossorigin(?=[ >])/g, '');
    }
  }],
  build: {
    outDir: resolve(import.meta.dirname, '../DiagramStudio/Sources/DiagramStudio/Resources/Web'),
    emptyOutDir: true,
    rollupOptions: { input: resolve(import.meta.dirname, 'index.html') }
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./tests/setup.js']
  }
});
