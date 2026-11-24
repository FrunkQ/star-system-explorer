// Builds the drawing bench (src/bench-drawing.ts) as one ES module for the static check page on /harness/
// (scripts/build-harness.sh). The page is plain JavaScript, so three.js is bundled in.
import { defineConfig } from "vite";

export default defineConfig({
  logLevel: "warn",
  build: {
    outDir: `${import.meta.dirname}/../../tests/harness/pkg/drawing`,
    emptyOutDir: true,
    target: "es2022",
    chunkSizeWarningLimit: 1200,
    lib: { entry: `${import.meta.dirname}/src/bench-drawing.ts`, formats: ["es"], fileName: () => "bench-drawing.js" },
  },
});
