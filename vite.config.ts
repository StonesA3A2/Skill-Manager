import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
    watch: {
      // Without this, Vite's watcher globs src-tauri/target too and races
      // Cargo's own writes to its build output — on Windows that file lock
      // conflict crashes the whole `tauri dev` process with EBUSY.
      ignored: ["**/src-tauri/**"],
    },
  },
  preview: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
  },
});
