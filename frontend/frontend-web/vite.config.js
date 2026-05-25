import checker from 'vite-plugin-checker';
import sourcemaps from 'rollup-plugin-sourcemaps';
import mkcert from 'vite-plugin-mkcert';
import fs from 'node:fs';
import sirv from 'sirv';
import mockApiPlugin from "vite-mock-api";

const ServerFilesPlugin = {
  name: 'serve-storage-files',
  configureServer(server) {
    const serverStatic = sirv('mock-data', {})
    server.middlewares.use(serverStatic);
  }
}

export default {
  base: "./",
  plugins: [
    // LoggerPlugin,
    mockApiPlugin(),
    mkcert({savePath: "../../test-cert/"}),
    checker({
      // e.g. use TypeScript check
      typescript: true,
    }),
    sourcemaps(),
    ServerFilesPlugin
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
    port: 5180,
    strictPort: true,
    https: true,
    proxy: {},
    headers:{
      "Cross-Origin-Embedder-Policy":"require-corp",
      "Cross-Origin-Resource-Policy":"cross-origin",
      "Cross-Origin-Opener-Policy":"same-origin"
    }
  }
}
