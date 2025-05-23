import { resolve, extname } from 'path'
import checker from 'vite-plugin-checker';
import sourcemaps from 'rollup-plugin-sourcemaps';
import fs from 'node:fs';
import dts from 'vite-plugin-dts'

export default {
  base: "./",
  define: {
    'process.env': {'NODE_ENV': 'production'}
  },
  plugins: [
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
  }
}
