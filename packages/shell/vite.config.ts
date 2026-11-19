import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  root: import.meta.dirname,
  base: "./",
  plugins: [svelte()],
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: "es2022",
    rollupOptions: { input: { main: `${import.meta.dirname}/index.html`, players: `${import.meta.dirname}/players.html` } },
  },
  server: { fs: { allow: ["../.."] } },
});
