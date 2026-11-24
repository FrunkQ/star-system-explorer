// The detail ladder's GPU node matches its processor twin (VP6), on WebGL2 always and on WebGPU where the browser
// has it. Builds tests/ui/ladder-gpu/ with Vite into a temporary folder, serves it, draws every noise kind into a
// float target and compares each pixel with ladderAt. Usage: node tests/ui/ladder-gpu.e2e.mjs
import { spawn } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { build } from "vite";

const { chromium } = createRequire(import.meta.url)("playwright");
const dir = mkdtempSync(join(tmpdir(), "ladder-gpu-"));
await build({ root: "tests/ui/ladder-gpu", base: "./", logLevel: "warn", build: { outDir: dir, emptyOutDir: true, chunkSizeWarningLimit: 4000 } });
const port = 9800 + Math.floor(Math.random() * 90);
const server = spawn(process.execPath, ["scripts/serve.mjs", dir, String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
let failed = false;
try {
  for (const [q, label, args, required] of [
    ["?gl", "WebGL2", [], true],
    ["", "WebGPU", ["--enable-unsafe-webgpu"], false],
  ]) {
    const browser = await chromium.launch({ args });
    const page = await browser.newPage();
    await page.goto(`http://localhost:${port}/index.html${q}`);
    await page.waitForFunction(() => window.__done, null, { timeout: 90000 });
    const out = await page.evaluate(() => window.__done);
    await browser.close();
    if (out.error || (label === "WebGPU" && out.backend !== "webgpu")) {
      if (required) throw new Error(`${label}: ${out.error ?? out.backend}`);
      console.log(`${label}: not available here (${out.error ? out.error.split("\n")[0] : out.backend}), skipped`);
      continue;
    }
    assert.ok(out.rungs >= 3, `${label}: only ${out.rungs} rungs`);
    for (const [kind, { maxErr, maxVal }] of Object.entries(out.results)) {
      // float32 on the GPU against f64: within a tenth of a per cent of the largest value.
      const ok = maxErr <= 1e-3 * Math.max(1, maxVal);
      console.log(`${ok ? "ok" : "FAIL"} ${label} ${kind}: largest difference ${maxErr.toExponential(2)} of ${maxVal.toFixed(3)}`);
      if (!ok) failed = true;
    }
  }
} finally {
  server.kill();
  rmSync(dir, { recursive: true, force: true });
}
if (failed) process.exit(1);
console.log("ladder GPU check passed");
