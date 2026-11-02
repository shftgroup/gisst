import { resolve } from "path";
import { defineConfig } from "vite";
import dts from "vite-plugin-dts";

export default defineConfig({
  build: {
    lib: {
      // Could also be a dictionary or array of multiple entry points
      entry: resolve(import.meta.dirname, "src/main.ts"),
      name: "ra-util",
      formats: ["es"],
      // the proper extensions will be added
      fileName: "ra-util",
    },
    outDir: "dist",
    sourcemap: true,
  },
  plugins: [
    dts({
      insertTypesEntry: true,
      copyDtsFiles: true,
    }),
  ],
  rollupOptions: {
    // make sure to externalize deps that shouldn't be bundled
    // into your library
    external: [],
    output: { globals: {} },
  },
});
