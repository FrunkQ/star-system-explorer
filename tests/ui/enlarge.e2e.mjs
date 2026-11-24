// VP5: enlarging on Light. Builds packages/renderer/test/enlarge-page.ts into a scratch folder (never the repo), runs
// it in Chromium on WebGL2 and, where the browser has it, WebGPU, and checks that a hard edge enlarged four times comes
// out narrower than plain bilinear, that sharpening narrows it further, and that neither rings past the picture's range.
import { build } from "vite";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import assert from "node:assert/strict";

const root = new URL("../../", import.meta.url).pathname;
const dir = mkdtempSync(join(tmpdir(), "sse-enlarge-"));
await build({ logLevel: "warn", configFile: false, build: { outDir: dir, emptyOutDir: true, target: "es2022", chunkSizeWarningLimit: 2000, lib: { entry: join(root, "packages/renderer/test/enlarge-page.ts"), formats: ["es"], fileName: () => "enlarge.js" } } });
const server = createServer((req, res) => {
  if (req.url === "/") return void res.writeHead(200, { "content-type": "text/html" }).end("<!doctype html><title>enlarge</title>");
  try {
    res.writeHead(200, { "content-type": "text/javascript" }).end(readFileSync(join(dir, "enlarge.js")));
  } catch {
    res.writeHead(404).end();
  }
}).listen(0);
const { chromium } = createRequire(import.meta.url)("playwright");
/** Pixels across the edge: those between 10% and 90%. */
const width = (xs) => xs.filter((v) => v > 0.1 && v < 0.9).length;
try {
  for (const args of [[], ["--enable-unsafe-webgpu"]]) {
    const b = await chromium.launch({ args });
    const p = await b.newPage();
    await p.goto(`http://127.0.0.1:${server.address().port}/`);
    const r = await p.evaluate(async () => (await import("/enlarge.js")).runEnlarge());
    await b.close();
    const w = { bilinear: width(r.bilinear), bicubic: width(r.bicubic), sharpened: width(r.sharpened) };
    console.log(`  ${r.backend}: edge ${w.bilinear} px bilinear, ${w.bicubic} px enlarged, ${w.sharpened} px sharpened`);
    assert.ok(w.bicubic < w.bilinear, `enlarged edge narrower than bilinear (${JSON.stringify(w)})`);
    assert.ok(w.sharpened <= w.bicubic, `sharpening does not widen it (${JSON.stringify(w)})`);
    for (const xs of [r.bicubic, r.sharpened]) assert.ok(Math.min(...xs) >= 0 && Math.max(...xs) <= 1, "no ringing past the range");
    assert.ok(r.sharpened[10] < 0.05 && r.sharpened[245] > 0.95, "flat ground stays flat");
  }
  console.log("enlarge e2e: ok");
} finally {
  server.close();
  rmSync(dir, { recursive: true, force: true });
}
