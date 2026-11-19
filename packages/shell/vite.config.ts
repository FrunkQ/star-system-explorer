import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { execSync } from "node:child_process";
import { readFileSync } from "node:fs";

// The app version players must match (V4): package version and commit.
function appVersion(): string {
  const v = JSON.parse(readFileSync(new URL("./package.json", import.meta.url), "utf8")).version as string;
  let commit = process.env.WORKERS_CI_COMMIT_SHA ?? process.env.GITHUB_SHA ?? "";
  if (!commit) {
    try {
      commit = execSync("git rev-parse HEAD", { cwd: import.meta.dirname }).toString().trim();
    } catch {
      commit = "local";
    }
  }
  return `${v}+${commit.slice(0, 10)}`;
}

export default defineConfig({
  root: import.meta.dirname,
  base: "./",
  plugins: [svelte()],
  define: { __SSE_APP__: JSON.stringify(appVersion()) },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: "es2022",
    // three.js (about 800 kB) is one lazily loaded chunk, fetched only when a 3D view starts its GPU path.
    chunkSizeWarningLimit: 900,
    rollupOptions: { input: { main: `${import.meta.dirname}/index.html`, players: `${import.meta.dirname}/players.html` } },
  },
  server: { fs: { allow: ["../.."] } },
});
