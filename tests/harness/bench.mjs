// The bench by script (PF2, handover/design/SSE4-performance-and-minimum-spec.md). Times every engine scenario in
// Node, in headless Chromium, and in Chromium with its CPU slowed four times (the phone stand-in GH0 used), and
// writes one sse.perf-report/1 per runner. Build first: scripts/build-harness.sh.
//
// Usage: node tests/harness/bench.mjs [--runners node,chromium,chromium-cpu4] [--runs 3] [--label <machine>]
//                                     [--out <dir>] [--against <dir>] [--only resolve,nav]
// --against <dir> compares with the reports from an earlier run in that folder (same runner names) and exits 1 when a
// scenario or a chunk is more than 5 % slower: ND3b's timing condition (SSE4-equation-block-design.md 8B). Run it
// on one machine, before and after the change; timings from different machines or busy runners do not compare.
import { createServer } from "node:http";
import { mkdirSync, readFileSync, writeFileSync, existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, join, extname, normalize } from "node:path";
import { createRequire } from "node:module";
import { cpus, totalmem } from "node:os";
import { benchReport, compareBench, runBench } from "./bench.js";

const here = dirname(fileURLToPath(import.meta.url));
const arg = (k, d) => {
  const i = process.argv.indexOf(`--${k}`);
  return i >= 0 ? process.argv[i + 1] : d;
};
const runners = arg("runners", "node,chromium,chromium-cpu4").split(",");
const runs = Number(arg("runs", "3"));
const machine = arg("label", "this machine");
const out = arg("out", null);
const against = arg("against", null);
const only = arg("only", null)?.split(",");
const doc = JSON.parse(readFileSync(join(here, "../../packages/ui-contract/src/perf-chunks.json"), "utf8"));
const today = new Date().toISOString().slice(0, 10);
const ms = (x) => (x >= 1000 ? `${(x / 1000).toFixed(2)} s` : `${x.toFixed(1)} ms`);

async function inNode() {
  const wasm = await import(join(here, "pkg/sse_wasm.js"));
  wasm.initSync({ module: readFileSync(join(here, "pkg/sse_wasm_bg.wasm")) });
  const results = await runBench(wasm, doc, { runs, only });
  const c = cpus();
  const device = { user_agent: `Node ${process.version}`, platform: process.platform, cores: c.length, cpu: c[0]?.model, memory_gb: Math.round(totalmem() / 2 ** 30), screen: "none", pixel_ratio: 1, gpu: { backend: "none" }, reduced_motion: false };
  return benchReport(results, doc, { device, label: `${machine}, Node`, today, runs });
}

async function inChromium(rate) {
  const { chromium } = createRequire(import.meta.url)("playwright");
  const types = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".json": "application/json", ".jsonl": "text/plain" };
  const server = createServer(async (req, res) => {
    const p = normalize(join(here, decodeURIComponent(new URL(req.url, "http://x").pathname)));
    const file = p.endsWith("/") ? join(p, "index.html") : p;
    if (!file.startsWith(here)) return void res.writeHead(403).end();
    try {
      res.writeHead(200, { "content-type": types[extname(file)] ?? "application/octet-stream" }).end(await readFile(file));
    } catch {
      res.writeHead(404).end();
    }
  }).listen(0);
  const browser = await chromium.launch();
  try {
    const page = await browser.newPage();
    if (rate > 1) await (await page.context().newCDPSession(page)).send("Emulation.setCPUThrottlingRate", { rate });
    await page.goto(`http://127.0.0.1:${server.address().port}/?bench&runs=${runs}`);
    await page.waitForFunction(() => document.body.dataset.bench === "done" || document.body.dataset.result === "error", null, { timeout: 30 * 60_000 });
    const rep = await page.evaluate(() => window.__benchReport);
    if (!rep) throw new Error("the page did not finish the bench");
    rep.label = `${machine}, Chromium ${browser.version()}${rate > 1 ? `, CPU slowed ${rate}×` : ""}`;
    rep.made_on = today;
    if (rate > 1) rep.device.note = `CPU slowed ${rate}× on the page's thread only: the kernel's speed and tier come from a worker, which Chromium does not slow`;
    if (only) rep.chunks = rep.chunks.map((c) => ({ ...c, sources: c.sources.filter((s) => only.includes(s.source.slice(6))) })).filter((c) => c.sources.length);
    return rep;
  } finally {
    await browser.close();
    server.close();
  }
}

let failed = false;
for (const runner of runners) {
  const rep = runner === "node" ? await inNode() : runner === "chromium" ? await inChromium(1) : runner === "chromium-cpu4" ? await inChromium(4) : null;
  if (!rep) throw new Error(`unknown runner ${runner}`);
  console.log(`\n${rep.label} (${runs} runs, median)`);
  for (const c of rep.chunks) console.log(`  ${c.name.padEnd(34)} ${ms(c.median_ms).padStart(10)}   ${c.sources.map((s) => `${s.source.slice(6)} ${ms(s.median_ms)}`).join(", ")}`);
  if (out) {
    mkdirSync(out, { recursive: true });
    writeFileSync(join(out, `${runner}.json`), JSON.stringify(rep, null, 2) + "\n");
  }
  if (against) {
    const f = join(against, `${runner}.json`);
    if (!existsSync(f)) {
      console.log(`  no earlier ${runner}.json in ${against} to compare with`);
      continue;
    }
    const { slower } = compareBench(JSON.parse(readFileSync(f, "utf8")), rep);
    if (slower.length) {
      failed = true;
      console.log(`  SLOWER than ${f} by more than 5 %:`);
      for (const r of slower) console.log(`    ${r.what}: ${ms(r.before_ms)} → ${ms(r.after_ms)} (+${(r.change * 100).toFixed(1)} %)`);
    } else console.log(`  within 5 % of ${f}`);
  }
}
process.exit(failed ? 1 : 0);
