import { resolve } from 'path'
import { defineConfig } from 'vite'
import dts from 'vite-plugin-dts'

export default defineConfig({
  build: {
    lib: {
      // Could also be a dictionary or array of multiple entry points
      entry: resolve(__dirname, 'src/main.ts'),
      name: 'embedv86',
      // the proper extensions will be added
      fileName: 'embedv86',
    },
    sourcemap:true,
    outDir:"dist",
  },
  plugins: [dts({skipDiagnostics:false,logDiagnostics:true,insertTypesEntry:true,copyDtsFiles:true,outputDir: ['dist', 'types'],})],
  rollupOptions: {
    // make sure to externalize deps that shouldn't be bundled
    // into your library
    external: [],
    output: {globals:{}},
  },
})

