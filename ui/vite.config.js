import { defineConfig } from "vite";

// `server.port`/`strictPort` must match src-tauri/tauri.conf.json devUrl.
export default defineConfig({
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    outDir: "dist",
    target: "es2022",
  },
});