import { resolve } from 'path'
import { defineConfig } from 'vite'
import dts from 'vite-plugin-dts'

export default defineConfig({
  build: {
    lib: {
      // Could also be a dictionary or array of multiple entry points
      entry: resolve(__dirname, 'src/ts/main.ts'),
      name: 'GisstPlayer',
      // the proper extensions will be added
      fileName: 'gisst-player',
    },
    outDir:"dist",
  },
  plugins: [dts({skipDiagnostics:false,logDiagnostics:true,insertTypesEntry:true,copyDtsFiles:true,outputDir: ['dist', 'types'],})],
  resolve: {
    alias: {
      '~bootstrap': resolve(__dirname, 'node_modules/bootstrap'),
    }
  },
  root: resolve(__dirname, 'src'),
  rollupOptions: {
    // make sure to externalize deps that shouldn't be bundled
    // into your library
    external: [],
    output: {globals:{}},
  },
})

