import { resolve, extname } from 'path'
import checker from 'vite-plugin-checker';
import sourcemaps from 'rollup-plugin-sourcemaps';
import mkcert from 'vite-plugin-mkcert';
import fs from 'node:fs';
import dts from 'vite-plugin-dts'

export default {
  base: "./",
  define: {
  'process.env': {'NODE_ENV': 'production'}
  },
  plugins: [
    mkcert({savePath: "../../test-cert/"}),
    checker({
      // e.g. use TypeScript check
      typescript: true,
    }),
    dts({skipDiagnostics:false,logDiagnostics:true,insertTypesEntry:true,copyDtsFiles:true,outputDir: ['dist', 'types'],}),
      sourcemaps()
  ],
  build: {
    lib: {
      entry: resolve(__dirname, 'src/main.ts'),
      name: 'UI',
      fileName: 'ui',
    },
    sourcemap: true,
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
