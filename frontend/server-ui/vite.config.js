import { resolve, extname } from 'path'
import checker from 'vite-plugin-checker';
import sourcemaps from 'rollup-plugin-sourcemaps';
import mkcert from 'vite-plugin-mkcert';
import fs from 'node:fs';
import dts from 'vite-plugin-dts';
import mockApiPlugin from "vite-mock-api";

export default {
  base: "./",
	define: { 'process.env.NODE_ENV': '"production"' },
  plugins: [
    mkcert({savePath: "../../test-cert/"}),
    checker({
      // e.g. use TypeScript check
      typescript: true,
    }),
    dts({skipDiagnostics:false,logDiagnostics:true,insertTypesEntry:true,copyDtsFiles:true,outputDir: ['dist', 'types'],}),
    sourcemaps(),
    mockApiPlugin()
  ],
  build: {
    lib: {
      entry: resolve(__dirname, 'src/ts/main.ts'),
      name: 'UI',
      formats: ['es'],
      fileName: 'ui',
    },
    sourcemap: true,
    rolldownOptions: {
      transform: { define: { 'process.env.NODE_ENV': "'production'" } }
    },
  },
  optimizeDeps: {
    entries: "build.rolldownOptions.input",
    force:true,
    rolldownOptions: {
      transform: { define: { 'process.env.NODE_ENV': "'production'" } }
    }
  },
  server: {
    https: true,
    host: '0.0.0.0',
    headers:{
      "Cross-Origin-Embedder-Policy":"require-corp",
      "Cross-Origin-Resource-Policy":"cross-origin",
      "Cross-Origin-Opener-Policy":"same-origin"
    }
  }
}
