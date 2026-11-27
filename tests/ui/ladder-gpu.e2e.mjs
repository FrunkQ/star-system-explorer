// The detail ladder's GPU node (VP6) and the texture ladder's bake (VP3) match their processor twins, on WebGL2 always and on WebGPU where the browser
// has it. Builds tests/ui/ladder-gpu/ with Vite into a temporary folder, serves it, draws every noise kind into a
// float target and compares each pixel with ladderAt. Usage: node tests/ui/ladder-gpu.e2e.mjs
import "./chart-first.mjs";
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
    // VP3: the baked cubes against the processor twin. Colour is linear 0..1; the normal's error is a length.
    const { colourErr, normalErr, tiltMax, lit, dark } = out.bake;
    const bakeOk = colourErr <= 0.03 && normalErr <= 0.02 && tiltMax > 0.003 && lit > 0 && dark === 0;
    console.log(`${bakeOk ? "ok" : "FAIL"} ${label} texture bake: colour within ${colourErr.toFixed(4)}, normal within ${normalErr.toFixed(4)} (largest true tilt ${(tiltMax * 180 / Math.PI).toFixed(2)}°); lit globe ${lit} pixels lit, ${dark} black or not a number`);
    if (!bakeOk) failed = true;
    // VP3 part 4: the deep patch read at texel centres. Colour 0..1 sRGB bytes; the normal is stored as half floats.
    const p = out.patch;
    const patchOk = p.texels > 2000 && p.colourErr <= 0.004 && p.weightErr <= 0.004 && p.normalErr <= 0.004 && p.tiltMax > 0.01;
    console.log(`${patchOk ? "ok" : "FAIL"} ${label} deep patch: ${p.texels} texels, colour within ${p.colourErr.toFixed(4)}, weight within ${p.weightErr.toFixed(4)}, normal within ${p.normalErr.toFixed(4)} (largest true tilt ${(p.tiltMax * 180 / Math.PI).toFixed(2)}°)`);
    if (!patchOk) failed = true;
  }
} finally {
  server.kill();
  rmSync(dir, { recursive: true, force: true });
}
if (failed) process.exit(1);
console.log("ladder GPU check passed");
