// The stirred face's GPU carrying (WS5, stirred-gpu.ts) matches its processor twin (stirred-carry.ts) on the kernel's
// own table, on WebGL2 always and on WebGPU where the browser has it. Builds tests/ui/stirred-gpu/ with Vite into a
// temporary folder, serves it, carries a 64-cell patch on Neptune both ways and compares where each cell is read.
// Usage: node tests/ui/stirred-gpu.e2e.mjs
import "./chart-first.mjs";
import { spawn } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";
import { build } from "vite";

const { chromium } = createRequire(import.meta.url)("playwright");
const dir = mkdtempSync(join(tmpdir(), "stirred-gpu-"));
await build({ root: "tests/ui/stirred-gpu", base: "./", logLevel: "warn", build: { outDir: dir, emptyOutDir: true, chunkSizeWarningLimit: 8000 } });
const port = 9700 + Math.floor(Math.random() * 90);
const server = spawn(process.execPath, ["scripts/serve.mjs", dir, String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
// Float32 against float64 over a thousand steps. Where the cloud was carried from agrees to a kilometre or so; where
// it is read is that place drifted along its row for the generation's age, so a carried latitude off by 0.001° (a
// float's worth over a thousand steps) moves the read by the jet's shear times the age: ten to thirty kilometres for
// the older generation, the processor's own float64 just as sensitive to its own last digits. The bound is a quarter
// of a 1024-wide face's pixel (Neptune's: about 38 km), far below what a picture shows.
const NEPTUNE_KM = 24764;
const LIMIT_KM = (2 * Math.PI * NEPTUNE_KM) / 1024 / 4;
const YOUNG_KM = 2;
let failed = false;
try {
  for (const [q, label, args, required] of [
    [`?gl${process.env.SSE_PROBE ? `&${process.env.SSE_PROBE}` : ""}`, "WebGL2", [], true],
    ["", "WebGPU", ["--enable-unsafe-webgpu"], false],
  ]) {
    const browser = await chromium.launch({ args });
    const page = await browser.newPage();
    if (process.env.SSE_DEBUG) page.on("console", (m) => console.log(`  [${label}] ${m.text().slice(0, Number(process.env.SSE_DEBUG_LEN ?? 600))}`));
    await page.goto(`http://localhost:${port}/index.html${q}`);
    let out;
    try {
      await page.waitForFunction(() => window.__done, null, { timeout: 900000 });
      out = await page.evaluate(() => window.__done);
    } catch (e) {
      // A software device can run out of memory building the shader (SwiftShader's WebGPU does, in a 16 GB container).
      out = { error: `the page crashed: ${String(e).split("\n")[0]}` };
    }
    await browser.close().catch(() => {});
    if (out.error || (label === "WebGPU" && out.backend !== "webgpu")) {
      if (required) throw new Error(`${label}: ${out.error ?? out.backend}`);
      console.log(`${label}: not available here (${out.error ? out.error.split("\n")[0] : out.backend}), skipped`);
      continue;
    }
    const worst = Math.max(...out.cells.flatMap((c) => c.km));
    const bad = out.cells.filter((c) => !(c.km[0] <= YOUNG_KM && c.km[1] <= LIMIT_KM));
    const young = Math.max(...out.cells.map((c) => c.km[0]));
    for (const c of bad.slice(0, 5)) console.log(`  cell ${c.k}: GPU ${c.gpu.map((x) => x.toFixed(3))} against ${c.cpu.map((x) => x.toFixed(3))} (${c.km.map((x) => x.toFixed(2))} km)`);
    const ok = bad.length === 0;
    console.log(`${ok ? "ok" : "FAIL"} ${label}: ${out.cells.length} cells carried, worst ${young.toFixed(2)} km (younger, limit ${YOUNG_KM}) and ${worst.toFixed(2)} km (both, limit ${LIMIT_KM.toFixed(1)}) from the processor's, in ${(out.ms / 1000).toFixed(1)} s`);
    if (!ok) failed = true;
  }
} finally {
  server.kill();
  rmSync(dir, { recursive: true, force: true });
}
if (failed) process.exit(1);
console.log("stirred GPU check passed");
