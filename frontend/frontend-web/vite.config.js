import checker from "vite-plugin-checker";
import mkcert from "vite-plugin-mkcert";
import fs from "node:fs";
import sirv from "sirv";
import mockApiPlugin from "@gisst/vite-mock-api";
import { defineConfig, esmExternalRequirePlugin } from "vite";
const ServerFilesPlugin = {
  name: "serve-storage-files",
  configureServer(server) {
    const serverStatic = sirv("mock-data", {});
    server.middlewares.use(serverStatic);
  },
};

export default defineConfig({
  base: "./",
  plugins: [
    esmExternalRequirePlugin({
      external: ["esbuild"],
    }),
    // LoggerPlugin,
    mockApiPlugin(),
    mkcert({ savePath: "../../test-cert/" }),
    checker({
      // e.g. use TypeScript check
      typescript: true,
    }),
    ServerFilesPlugin,
  ],
  build: {
    sourcemap: true,
    rollupOptions: {
      output: {
        entryFileNames: `assets/[name].js`,
        chunkFileNames: `assets/[name].js`,
        assetFileNames: `assets/[name].[ext]`,
      },
    },
  },
  server: {
    port: 5180,
    strictPort: true,
    https: true,
    proxy: {},
    headers: {
      "Cross-Origin-Embedder-Policy": "require-corp",
      "Cross-Origin-Resource-Policy": "cross-origin",
      "Cross-Origin-Opener-Policy": "same-origin",
    },
  },
});
