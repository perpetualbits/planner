// vite.config.ts — Vite configuration for the hub-tauri frontend.
//
// Uses vite-plugin-solid for SolidJS JSX transformation.
// The devUrl (http://localhost:5173) must match tauri.conf.json build.devUrl.

import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

export default defineConfig({
  plugins: [solid()],
  // Make Vite aware it is running inside Tauri so it can use the correct
  // base path and avoid conflicts with the Tauri protocol.
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    // Tauri's bundler expects the frontend output in dist/.
    outDir: "dist",
    target: ["es2021", "chrome105", "safari15"],
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
