// The settle kit (FL2, packages/renderer/src/settle-kit.ts) matches its processor twins (settle-kit-cpu.ts) on a
// 64-wide patch, on WebGL2 always and on WebGPU where the browser has it. Builds tests/ui/settle-kit/ with Vite into a
// temporary folder, serves it, draws Carry, Exposure and Fade both ways and compares.
// Usage: node tests/ui/settle-kit.e2e.mjs [--bench]
import "./chart-first.mjs";
import { spawn } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";
import { build } from "vite";

const { chromium } = createRequire(import.meta.url)("playwright");
const bench = process.argv.includes("--bench");
// --screens <dir>: also draw a banded face carried and exposed, and save a picture of it per backend.
const screens = process.argv.includes("--screens") ? process.argv[process.argv.indexOf("--screens") + 1] : null;
// SSE_CHANNEL=chrome runs the installed Chrome (a real GPU, as on AC-ProArt) in place of Playwright's Chromium.
const channel = process.env.SSE_CHANNEL;
const dir = mkdtempSync(join(tmpdir(), "settle-kit-"));
await build({ root: "tests/ui/settle-kit", base: "./", logLevel: "warn", build: { outDir: dir, emptyOutDir: true, chunkSizeWarningLimit: 8000 } });
const port = 9800 + Math.floor(Math.random() * 90);
const server = spawn(process.execPath, ["scripts/serve.mjs", dir, String(port)], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 400));
// The design's bound (8A) is one pixel of its twin. Held here to a hundredth of one, since float32 against float64
// agrees to about 1e-4 of a pixel (SwiftShader, 28 Nov) and a whole pixel let a loop's turning be left out unseen
// (tried). `pixels` is the worst difference over the test picture's steepest change in a pixel; the zero carry and the
// fade's ends must be equal to the bit.
const PIXEL = 0.01;
let failed = false;
try {
  for (const [q, label, args, required] of [
    [`?gl${bench ? "&bench" : ""}${screens ? "&show" : ""}`, "WebGL2", [], true],
    [`?x${bench ? "&bench" : ""}${screens ? "&show" : ""}`, "WebGPU", ["--enable-unsafe-webgpu"], false],
  ]) {
    const browser = await chromium.launch({ args, ...(channel ? { channel } : {}) });
    const page = await browser.newPage({ viewport: { width: 1600, height: 900 } });
    if (process.env.SSE_DEBUG) page.on("console", (m) => console.log(`  [${label}] ${m.text()}`));
    await page.goto(`http://localhost:${port}/index.html${q}`);
    let out;
    try {
      await page.waitForFunction(() => window.__done, null, { timeout: 300000 });
      out = await page.evaluate(() => window.__done);
    } catch (e) {
      out = { error: `the page crashed: ${String(e).split("\n")[0]}` };
    }
    if (screens && !out.error) {
      const { mkdirSync } = await import("node:fs");
      mkdirSync(screens, { recursive: true });
      await page.screenshot({ path: join(screens, `settle-kit-${label.toLowerCase()}.png`), fullPage: true }).catch(() => {});
    }
    await browser.close().catch(() => {});
    if (out.error || (label === "WebGPU" && out.backend !== "webgpu")) {
      if (required) throw new Error(`${label}: ${out.error ?? out.backend}`);
      console.log(`${label}: not available here (${out.error ? out.error.split("\n")[0] : out.backend}), skipped`);
      continue;
    }
    for (const c of out.cases) {
      const ok = c.pixels <= PIXEL && c.exact !== false;
      if (!ok) failed = true;
      console.log(`${ok ? "ok" : "FAIL"} ${label} ${c.name}: worst ${c.worst.toFixed(4)} (${c.pixels.toFixed(4)} px, limit ${PIXEL})${c.exact === undefined ? "" : c.exact ? ", equal to the bit" : ", NOT equal to the bit"}`);
    }
    if (out.bench) for (const b of out.bench) console.log(`  bench ${label} ${b.piece} ${b.width}x${b.height}: ${b.ms} ms (with upload and read-back)`);
  }
} finally {
  server.kill();
  rmSync(dir, { recursive: true, force: true });
}
if (failed) process.exit(1);
console.log("settle kit check passed");
