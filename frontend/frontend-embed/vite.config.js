import { resolve, extname } from 'path'
import checker from 'vite-plugin-checker';
import sourcemaps from 'rollup-plugin-sourcemaps';
import dts from 'vite-plugin-dts'
import mkcert from 'vite-plugin-mkcert';

export default {
  base: "./",
  plugins: [
    mkcert({savePath: "../../test-cert/"}),
    checker({
      typescript: true,
    }),
    dts({skipDiagnostics:false,logDiagnostics:true,insertTypesEntry:true,copyDtsFiles:true,outputDir: ['dist', 'types'],}),
    sourcemaps()
  ],
  build: {
    lib: {
      entry: resolve(__dirname, 'src/main.ts'),
      name: 'GISST',
      fileName: 'embed',
    },
    sourcemap: true,
  },
  server: {
    headers:{
      "Cross-Origin-Embedder-Policy":"require-corp",
      "Cross-Origin-Resource-Policy":"cross-origin",
      "Cross-Origin-Opener-Policy":"same-origin",
      "Access-Control-Allow-Origin":"*",
      "Content-Security-Policy": "script-src 'self' 'unsafe-inline' blob: 'wasm-unsafe-eval' https://localhost:3000/; worker-src 'self' blob: https://localhost:3000/;"
    },
  },
}
