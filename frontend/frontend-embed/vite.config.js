import { resolve, extname } from 'path'
import checker from 'vite-plugin-checker';
import sourcemaps from 'rollup-plugin-sourcemaps';
import dts from 'vite-plugin-dts'
import mkcert from 'vite-plugin-mkcert';
import sirv from 'sirv';

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
    mkcert({savePath: "../../test-cert/"}),
    checker({
      typescript: true,
    }),
    dts({skipDiagnostics:false,logDiagnostics:true,insertTypesEntry:true,copyDtsFiles:true,outputDir: ['dist', 'types'],}),
    sourcemaps(),
    ServerFilesPlugin,
  ],
  build: {
    lib: {
      entry: resolve(__dirname, 'src/main.ts'),
      name: 'GISST',
      fileName: 'embed',
      formats: ['es'],
    },
    sourcemap: true,
  },
  server: {
    port: 5177,
    strictPort: true,
    headers:{
      "Cross-Origin-Embedder-Policy":"require-corp",
      "Cross-Origin-Resource-Policy":"cross-origin",
      "Cross-Origin-Opener-Policy":"same-origin",
      "Access-Control-Allow-Origin":"*",
      "Content-Security-Policy": "script-src 'self' 'unsafe-inline' blob: 'wasm-unsafe-eval'; worker-src 'self' blob: ;"
    },
  },
}
