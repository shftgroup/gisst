import { resolve } from 'path'
import checker from 'vite-plugin-checker';
import sourcemaps from 'rollup-plugin-sourcemaps';
import dts from 'vite-plugin-dts'
import mkcert from 'vite-plugin-mkcert';

export default {
  plugins: [
    mkcert(),
    checker({
      // e.g. use TypeScript check
      typescript: true,
    }),
    dts({skipDiagnostics:false,logDiagnostics:true,insertTypesEntry:true,copyDtsFiles:true,outputDir: ['dist', 'types'],}),
    sourcemaps()
  ],
  build: {
    lib: {
      // Could also be a dictionary or array of multiple entry points
      entry: resolve(__dirname, 'src/main.ts'),
      name: 'GISST',
      // the proper extensions will be added
      fileName: 'embed',
    },
    sourcemap: true,
    // rollupOptions: {
    //   output: {
    //     entryFileNames: `embed/[name].js`,
    //     chunkFileNames: `embed/[name].js`,
    //     assetFileNames: `embed/[name].[ext]`
    //   }
    // }
  },
  server: {
    headers:{
      "Cross-Origin-Embedder-Policy":"require-corp",
      "Cross-Origin-Resource-Policy":"cross-origin",
      "Cross-Origin-Opener-Policy":"same-origin",
      "Access-Control-Allow-Origin":"*",
      "Content-Security-Policy": "script-src 'self' 'unsafe-inline' blob: 'wasm-unsafe-eval' https://localhost:3000/; worker-src 'self' blob: https://localhost:3000/;"
    },
  }
}
