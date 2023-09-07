// vite.config.js
import { resolve } from "path";
import { defineConfig } from "file:///Users/eric.kaltman/Dropbox/Reference/Development/projects/csuci/neh-gisst-2023-2024/embedulator/frontend/node_modules/vite/dist/node/index.js";
import dts from "file:///Users/eric.kaltman/Dropbox/Reference/Development/projects/csuci/neh-gisst-2023-2024/embedulator/frontend/node_modules/vite-plugin-dts/dist/index.mjs";
var __vite_injected_original_dirname = "/Users/eric.kaltman/Dropbox/Reference/Development/projects/csuci/neh-gisst-2023-2024/embedulator/frontend/gisst-player-ui";
var htmlImport = {
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
      code = `export default \`${code}\``;
    }
    return { code, map: null };
  }
};
var vite_config_default = defineConfig({
  build: {
    lib: {
      // Could also be a dictionary or array of multiple entry points
      entry: resolve(__vite_injected_original_dirname, "src/ts/main.ts"),
      name: "GisstPlayer",
      // the proper extensions will be added
      fileName: "gisst-player"
    },
    sourcemap: true,
    outDir: "dist"
  },
  plugins: [
    dts({ skipDiagnostics: false, logDiagnostics: true, insertTypesEntry: true, copyDtsFiles: true, outputDir: ["dist", "types"] }),
    htmlImport
  ],
  resolve: {
    alias: {
      "~bootstrap": resolve(__vite_injected_original_dirname, "../node_modules/bootstrap")
    }
  },
  // root: resolve(__dirname, 'src'),
  rollupOptions: {
    // make sure to externalize deps that shouldn't be bundled
    // into your library
    external: [],
    output: { globals: {} }
  }
});
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcuanMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCIvVXNlcnMvZXJpYy5rYWx0bWFuL0Ryb3Bib3gvUmVmZXJlbmNlL0RldmVsb3BtZW50L3Byb2plY3RzL2NzdWNpL25laC1naXNzdC0yMDIzLTIwMjQvZW1iZWR1bGF0b3IvZnJvbnRlbmQvZ2lzc3QtcGxheWVyLXVpXCI7Y29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2ZpbGVuYW1lID0gXCIvVXNlcnMvZXJpYy5rYWx0bWFuL0Ryb3Bib3gvUmVmZXJlbmNlL0RldmVsb3BtZW50L3Byb2plY3RzL2NzdWNpL25laC1naXNzdC0yMDIzLTIwMjQvZW1iZWR1bGF0b3IvZnJvbnRlbmQvZ2lzc3QtcGxheWVyLXVpL3ZpdGUuY29uZmlnLmpzXCI7Y29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2ltcG9ydF9tZXRhX3VybCA9IFwiZmlsZTovLy9Vc2Vycy9lcmljLmthbHRtYW4vRHJvcGJveC9SZWZlcmVuY2UvRGV2ZWxvcG1lbnQvcHJvamVjdHMvY3N1Y2kvbmVoLWdpc3N0LTIwMjMtMjAyNC9lbWJlZHVsYXRvci9mcm9udGVuZC9naXNzdC1wbGF5ZXItdWkvdml0ZS5jb25maWcuanNcIjtpbXBvcnQgeyByZXNvbHZlIH0gZnJvbSAncGF0aCdcbmltcG9ydCB7IGRlZmluZUNvbmZpZyB9IGZyb20gJ3ZpdGUnXG5pbXBvcnQgZHRzIGZyb20gJ3ZpdGUtcGx1Z2luLWR0cydcblxuY29uc3QgaHRtbEltcG9ydCA9IHtcbiAgbmFtZTogXCJodG1sSW1wb3J0XCIsXG4gIC8qKlxuICAgKiBDaGVja3MgdG8gZW5zdXJlIHRoYXQgYSBodG1sIGZpbGUgaXMgYmVpbmcgaW1wb3J0ZWQuXG4gICAqIElmIGl0IGlzIHRoZW4gaXQgYWx0ZXJzIHRoZSBjb2RlIGJlaW5nIHBhc3NlZCBhcyBiZWluZyBhIHN0cmluZyBiZWluZyBleHBvcnRlZCBieSBkZWZhdWx0LlxuICAgKiBAcGFyYW0ge3N0cmluZ30gY29kZSBUaGUgZmlsZSBhcyBhIHN0cmluZy5cbiAgICogQHBhcmFtIHtzdHJpbmd9IGlkIFRoZSBhYnNvbHV0ZSBwYXRoLlxuICAgKiBAcmV0dXJucyB7e2NvZGU6IHN0cmluZ319XG4gICAqL1xuICB0cmFuc2Zvcm0oY29kZSwgaWQpIHtcbiAgICBpZiAoL14uKlxcLmh0bWwkL2cudGVzdChpZCkpIHtcbiAgICAgIGNvZGUgPSBgZXhwb3J0IGRlZmF1bHQgXFxgJHtjb2RlfVxcYGBcbiAgICB9XG4gICAgcmV0dXJuIHsgY29kZSwgbWFwOm51bGwgfVxuICB9XG59XG5leHBvcnQgZGVmYXVsdCBkZWZpbmVDb25maWcoe1xuICBidWlsZDoge1xuICAgIGxpYjoge1xuICAgICAgLy8gQ291bGQgYWxzbyBiZSBhIGRpY3Rpb25hcnkgb3IgYXJyYXkgb2YgbXVsdGlwbGUgZW50cnkgcG9pbnRzXG4gICAgICBlbnRyeTogcmVzb2x2ZShfX2Rpcm5hbWUsICdzcmMvdHMvbWFpbi50cycpLFxuICAgICAgbmFtZTogJ0dpc3N0UGxheWVyJyxcbiAgICAgIC8vIHRoZSBwcm9wZXIgZXh0ZW5zaW9ucyB3aWxsIGJlIGFkZGVkXG4gICAgICBmaWxlTmFtZTogJ2dpc3N0LXBsYXllcicsXG4gICAgfSxcbiAgICBzb3VyY2VtYXA6dHJ1ZSxcbiAgICBvdXREaXI6XCJkaXN0XCJcbiAgfSxcbiAgcGx1Z2luczogW1xuICAgICAgZHRzKHtza2lwRGlhZ25vc3RpY3M6ZmFsc2UsbG9nRGlhZ25vc3RpY3M6dHJ1ZSxpbnNlcnRUeXBlc0VudHJ5OnRydWUsY29weUR0c0ZpbGVzOnRydWUsb3V0cHV0RGlyOiBbJ2Rpc3QnLCAndHlwZXMnXSx9KSxcbiAgICAgIGh0bWxJbXBvcnRcbiAgXSxcbiAgcmVzb2x2ZToge1xuICAgIGFsaWFzOiB7XG4gICAgICAnfmJvb3RzdHJhcCc6IHJlc29sdmUoX19kaXJuYW1lLCAnLi4vbm9kZV9tb2R1bGVzL2Jvb3RzdHJhcCcpLFxuICAgIH1cbiAgfSxcbiAgLy8gcm9vdDogcmVzb2x2ZShfX2Rpcm5hbWUsICdzcmMnKSxcbiAgcm9sbHVwT3B0aW9uczoge1xuICAgIC8vIG1ha2Ugc3VyZSB0byBleHRlcm5hbGl6ZSBkZXBzIHRoYXQgc2hvdWxkbid0IGJlIGJ1bmRsZWRcbiAgICAvLyBpbnRvIHlvdXIgbGlicmFyeVxuICAgIGV4dGVybmFsOiBbXSxcbiAgICBvdXRwdXQ6IHtnbG9iYWxzOnt9fSxcbiAgfSxcbn0pXG5cbiJdLAogICJtYXBwaW5ncyI6ICI7QUFBNmhCLFNBQVMsZUFBZTtBQUNyakIsU0FBUyxvQkFBb0I7QUFDN0IsT0FBTyxTQUFTO0FBRmhCLElBQU0sbUNBQW1DO0FBSXpDLElBQU0sYUFBYTtBQUFBLEVBQ2pCLE1BQU07QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBLEVBUU4sVUFBVSxNQUFNLElBQUk7QUFDbEIsUUFBSSxjQUFjLEtBQUssRUFBRSxHQUFHO0FBQzFCLGFBQU8sb0JBQW9CLElBQUk7QUFBQSxJQUNqQztBQUNBLFdBQU8sRUFBRSxNQUFNLEtBQUksS0FBSztBQUFBLEVBQzFCO0FBQ0Y7QUFDQSxJQUFPLHNCQUFRLGFBQWE7QUFBQSxFQUMxQixPQUFPO0FBQUEsSUFDTCxLQUFLO0FBQUE7QUFBQSxNQUVILE9BQU8sUUFBUSxrQ0FBVyxnQkFBZ0I7QUFBQSxNQUMxQyxNQUFNO0FBQUE7QUFBQSxNQUVOLFVBQVU7QUFBQSxJQUNaO0FBQUEsSUFDQSxXQUFVO0FBQUEsSUFDVixRQUFPO0FBQUEsRUFDVDtBQUFBLEVBQ0EsU0FBUztBQUFBLElBQ0wsSUFBSSxFQUFDLGlCQUFnQixPQUFNLGdCQUFlLE1BQUssa0JBQWlCLE1BQUssY0FBYSxNQUFLLFdBQVcsQ0FBQyxRQUFRLE9BQU8sRUFBRSxDQUFDO0FBQUEsSUFDckg7QUFBQSxFQUNKO0FBQUEsRUFDQSxTQUFTO0FBQUEsSUFDUCxPQUFPO0FBQUEsTUFDTCxjQUFjLFFBQVEsa0NBQVcsMkJBQTJCO0FBQUEsSUFDOUQ7QUFBQSxFQUNGO0FBQUE7QUFBQSxFQUVBLGVBQWU7QUFBQTtBQUFBO0FBQUEsSUFHYixVQUFVLENBQUM7QUFBQSxJQUNYLFFBQVEsRUFBQyxTQUFRLENBQUMsRUFBQztBQUFBLEVBQ3JCO0FBQ0YsQ0FBQzsiLAogICJuYW1lcyI6IFtdCn0K
