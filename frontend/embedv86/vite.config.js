import { resolve } from 'path'
import { defineConfig } from 'vite'
import dts from 'vite-plugin-dts'
import serveStatic from 'serve-static';

const ServerFilesPlugin = {
    name: 'serve-server-files',
    configureServer(server) {
      const serverStatic = serveStatic('../../', {acceptRanges:true, cacheControl:false,index:false})
        server.middlewares.use(serverStatic);
    }
}
export default defineConfig({
  build: {
    lib: {
      // Could also be a dictionary or array of multiple entry points
      entry: resolve(__dirname, 'src/main.ts'),
      formats: ['es'],
      name: 'embedv86',
      // the proper extensions will be added
      fileName: 'embedv86',
    },
    sourcemap:true,
    outDir:"dist",
  },
  plugins: [
    dts({skipDiagnostics:false,logDiagnostics:true,insertTypesEntry:true,copyDtsFiles:true,outputDir: ['dist', 'types'],}),
    ServerFilesPlugin
  ],
  test: {
    includeSource: ['src/**/*.{js,ts}'],
  },
  define: {
    'import.meta.vitest': 'undefined'
  }
})

