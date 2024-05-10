import { defineConfig } from "vite";

import cliFlagsPlugin from "./cli-flags.plugin.js";

export default defineConfig({
  plugins: [
    cliFlagsPlugin(),
  ],
  root: new URL("./web", import.meta.url).pathname,
  css: {
    modules: {
      localsConvention: "camelCase",
    },
  },
  build: {
    target: "esnext",
    outDir: new URL("./dist", import.meta.url).pathname,
    emptyOutDir: false,
    minify: "esbuild",
  },
  resolve: {
    alias: {
      "react": "preact/compat",
      "react-dom": "preact/compat",
    },
  },
  ssr: {
    noExternal: true,
  },
});
