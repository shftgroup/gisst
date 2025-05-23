import { resolve } from 'path'
import { defineConfig } from 'vite'
import dts from 'vite-plugin-dts'

const htmlImport = {
  name: "htmlImport",
  /**
   * Checks to ensure that a html file is being imported.
   * If it is then it alters the code being passed as being a string being exported by default.
   * @param {string} code The file as a string.
   * @param {string} id The absolute path.
   * @returns {{code: string}}
   */
  transform(code, id) {
    if (/^.*\.html$/g.test(id)) {
      code = `export default \`${code}\``
    }
    return { code, map:null }
  }
}
export default defineConfig({
  css: {
    preprocessorOptions: {
      scss: {
        quietDeps: true
      }
    }
  },
  build: {
    lib: {
      // Could also be a dictionary or array of multiple entry points
      entry: resolve(__dirname, 'src/ts/main.ts'),
      name: 'GisstPlayer',
      // the proper extensions will be added
      fileName: 'gisst-player',
    },
    sourcemap:true,
    outDir:"dist"
  },
  plugins: [
      dts({skipDiagnostics:false,logDiagnostics:true,insertTypesEntry:true,copyDtsFiles:true,outputDir: ['dist', 'types'],}),
      htmlImport
  ],
  resolve: {
    alias: {
      '~bootstrap': resolve(__dirname, '../node_modules/bootstrap'),
      '~keyboard-css': resolve(__dirname, '../node_modules/keyboard-css'),
    }
  },
  // root: resolve(__dirname, 'src'),
  rollupOptions: {
    // make sure to externalize deps that shouldn't be bundled
    // into your library
    external: [],
    output: {globals:{}},
  },
})
