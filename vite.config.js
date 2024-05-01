import { defineConfig } from "vite";

export default defineConfig({
  root: new URL("./web", import.meta.url).pathname,
  build: {
    target: "esnext",
    outDir: new URL("./dist", import.meta.url).pathname,
    emptyOutDir: false,
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
