import { defineConfig } from "vite";

export default defineConfig({
  root: new URL("./web", import.meta.url).pathname,
  build: {
    outDir: new URL("./dist", import.meta.url).pathname,
    emptyOutDir: true,
  },
  resolve: {
    alias: {
      "react": "preact/compat",
      "react-dom": "preact/compat",
    },
  },
  esbuild: {
    jsxInject: `import * as React from "react";`,
  },
});
