import checker from 'vite-plugin-checker';
import { defineConfig } from 'vite';

// https://vitejs.dev/config
export default defineConfig({
  plugins: [
    checker({
      // e.g. use TypeScript check
      typescript: true,
    }),
  ],
  build: {
    sourcemap:true,
    rollupOptions: {
      output: {
        entryFileNames: `[name].js`,
        chunkFileNames: `[name].js`,
        assetFileNames: `[name].[ext]`
      }
    }
  },
});
