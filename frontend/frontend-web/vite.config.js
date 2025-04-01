import checker from 'vite-plugin-checker';
import sourcemaps from 'rollup-plugin-sourcemaps';
import mkcert from 'vite-plugin-mkcert';
import fs from 'node:fs';

export default {
  base: "./",
  plugins: [
    mkcert({savePath: "../../test-cert/"}),
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
      "Cross-Origin-Resource-Policy":"cross-origin",
      "Cross-Origin-Opener-Policy":"same-origin"
    },
    proxy: {
      "/storage": {
        changeOrigin: false,
        secure: false,
        target: {
          protocol: 'https:',
          host: 'localhost',
          port: 3000,
        },
      }
    }
  }
}
