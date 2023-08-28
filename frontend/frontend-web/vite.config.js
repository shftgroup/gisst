import checker from 'vite-plugin-checker';
import sourcemaps from 'rollup-plugin-sourcemaps';

export default {
  plugins: [
    checker({
      // e.g. use TypeScript check
      typescript: true,
    }),
      sourcemaps()
  ],
  build: {
    sourcemap: true,
    rollupOptions: {
      output: {
        entryFileNames: `assets/[name].js`,
        chunkFileNames: `assets/[name].js`,
        assetFileNames: `assets/[name].[ext]`
      }
    }
  },
  server: {
    headers:{
      "Cross-Origin-Embedder-Policy":"require-corp",
      "Cross-Origin-Opener-Policy":"same-origin"
    },
    proxy: {
      "/storage": "http://localhost:3000/"
    }
  }
}
