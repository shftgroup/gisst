import { resolve } from "path";
import { defineConfig } from "vite";
import dts from "vite-plugin-dts";

export default defineConfig({
  server: {
    host: true,
    port: 5173,
    open: false,
  },
  build: {
    lib: {
      entry: resolve(__dirname, "src/ts/main.ts"),
      name: "EditorUi",
      formats: ["es"],
      fileName: "editor-ui",
    },
    outDir: "dist",
  },
  plugins: [dts()],
});
