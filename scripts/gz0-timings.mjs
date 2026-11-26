// GZ0, the timings half (design/SSE4-grand-zoom-design.md section 10): the cold cost of every read the grand zoom
// would make, in Node, in headless Chromium, and in Chromium with its CPU slowed four times (the phone stand-in the
// bench uses). Checks only: it changes nothing and no test reads it. Build the wasm first (scripts/build-wasm.sh or
// scripts/build-harness.sh), then:
//
//   node scripts/gz0-timings.mjs [--runners node,chromium,chromium-cpu4] [--runs 3] [--out <file.json>]   (the 26 Nov run: handover/packages/GZ0-timings-2026-11-26.json)
//
// Each read runs on a fresh kernel (the four base packs loaded and a universe opened, timed apart as "setup"), once
// cold and then once more warm on the same kernel. Times are medians over the runs. The kernel runs on the page's
// own thread here, so the CPU slowdown applies to it (in the app it runs in a worker, which Chromium does not slow).
import { createServer } from "node:http";
import { readFileSync, writeFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { join, extname, normalize } from "node:path";
import { createRequire } from "node:module";
import { cpus } from "node:os";

const root = fileURLToPath(new URL("../", import.meta.url));
const arg = (k, d) => {
  const i = process.argv.indexOf(`--${k}`);
  return i >= 0 ? process.argv[i + 1] : d;
};
const runners = arg("runners", "node,chromium,chromium-cpu4").split(",");
const runs = Number(arg("runs", "3"));
const out = arg("out", null);

const PACKS = ["base-physics", "base-time", "base-reference", "base-manifest"].map((p) => `packs/${p}/pack.json`);
const UNIVERSE = { op: "open_universe", id: "gz0", seed: "gz0 timings" };

// The reads of section 10, as the views ask for them today. `then` makes a read from an earlier answer (the biggest
// neighbour in the home cell, for its full chart).
const READS = [
  { id: "web-cube-48-100", rung: "web", what: "Web cube, 48³ over 100 Mpc", ask: { op: "web_slice", cube: true, extent_mpc: 100, n: 48 } },
  { id: "web-galaxies-48-100", rung: "web", what: "Web cube with its galaxies, 48³ over 100 Mpc", ask: { op: "web_slice", cube: true, haloes: true, extent_mpc: 100, n: 48, galaxy_floor_msun: 1e11 } },
  { id: "web-galaxies-48-200", rung: "web", what: "Web cube with its galaxies, 48³ over 200 Mpc (the view's default)", ask: { op: "web_slice", cube: true, haloes: true, extent_mpc: 200, n: 48, galaxy_floor_msun: 8e11 } },
  { id: "web-galaxies-32-100", rung: "web", what: "Web cube with its galaxies, 32³ over 100 Mpc (the basic tier's)", ask: { op: "web_slice", cube: true, haloes: true, extent_mpc: 100, n: 32, galaxy_floor_msun: 1e11 } },
  { id: "halo-cell", rung: "cluster", what: "One 8 Mpc halo cell (Galaxies nearby today)", ask: { op: "halo_cell", cell: [0, 0, 0], min_mass_msun: 1e10 } },
  { id: "neighbourhood", rung: "cluster", what: "The neighbourhood's galaxies and home's satellites (galaxies_seen and satellites_seen, through the sky)", ask: { op: "sky_from_here", stars: false, galaxies: true, limit_mag: 30 } },
  { id: "home-chart", rung: "galaxy", what: "The home galaxy's chart", ask: { op: "galaxy_chart" } },
  {
    id: "neighbour-chart",
    rung: "cluster",
    what: "One neighbour's full chart (the biggest other halo in the home cell)",
    first: { op: "halo_cell", cell: [0, 0, 0], min_mass_msun: 1e11 },
    then: (r) => {
      const hs = (r.cell?.haloes ?? []).slice().sort((a, b) => b.mass_msun - a.mass_msun);
      const h = hs[1] ?? hs[0];
      return h ? { op: "galaxy_chart", cell: [0, 0, 0], halo: h.address } : null;
    },
  },
  { id: "star-cell", rung: "stars", what: "The home star cell (10 pc)", ask: { op: "star_cell" } },
];

function runAll(Kernel, now, packs) {
  const call = (k, req) => JSON.parse(k.handle(JSON.stringify(req)));
  const fresh = () => {
    const t0 = now();
    const k = new Kernel();
    const st = call(k, { op: "load_stack", packs });
    if (st.type !== "stack_loaded") throw new Error(`stack: ${JSON.stringify(st).slice(0, 300)}`);
    const u = call(k, UNIVERSE);
    if (u.type !== "universe_opened") throw new Error(`universe: ${JSON.stringify(u).slice(0, 300)}`);
    return { k, setup: now() - t0 };
  };
  const med = (xs) => [...xs].sort((a, b) => a - b)[Math.floor((xs.length - 1) / 2)];
  const results = [];
  for (const r of READS) {
    const cold = [];
    const warm = [];
    const setup = [];
    let bytes = 0;
    let error = null;
    for (let i = 0; i < runs && !error; i++) {
      const { k, setup: s } = fresh();
      setup.push(s);
      let ask = r.ask;
      if (r.then) ask = r.then(call(k, r.first));
      if (!ask) {
        error = "nothing to ask";
        break;
      }
      const t0 = now();
      const text = k.handle(JSON.stringify(ask));
      cold.push(now() - t0);
      const reply = JSON.parse(text);
      if (reply.type === "error" || reply.code) error = `${reply.code ?? ""} ${reply.message ?? ""}`.trim();
      bytes = text.length;
      const t1 = now();
      k.handle(JSON.stringify(ask));
      warm.push(now() - t1);
      k.free?.();
    }
    results.push({ id: r.id, rung: r.rung, what: r.what, cold_ms: cold.length ? med(cold) : null, warm_ms: warm.length ? med(warm) : null, setup_ms: setup.length ? med(setup) : null, reply_kb: Math.round(bytes / 1024), ...(error ? { error } : {}) });
  }
  return results;
}

async function inNode() {
  const wasm = await import(join(root, "packages/kernel-client/wasm/sse_wasm.js"));
  wasm.initSync({ module: readFileSync(join(root, "packages/kernel-client/wasm/sse_wasm_bg.wasm")) });
  const packs = PACKS.map((p) => JSON.parse(readFileSync(join(root, p), "utf8")));
  return { runner: `Node ${process.version}`, results: runAll(wasm.Kernel, () => performance.now(), packs) };
}

async function inChromium(rate) {
  const { chromium } = createRequire(import.meta.url)("playwright");
  const types = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".json": "application/json" };
  const allowed = ["packages/kernel-client/wasm/", "packs/"];
  const server = createServer(async (req, res) => {
    const path = decodeURIComponent(new URL(req.url, "http://x").pathname).slice(1);
    if (path === "" || path === "gz0.html") return void res.writeHead(200, { "content-type": "text/html" }).end("<!doctype html><title>GZ0</title>");
    const file = normalize(join(root, path));
    if (!file.startsWith(root) || !allowed.some((a) => path.startsWith(a))) return void res.writeHead(403).end();
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
    await page.goto(`http://127.0.0.1:${server.address().port}/gz0.html`);
    const results = await page.evaluate(
      async ({ src, packsAt, runs: n }) => {
        const mod = await import("/packages/kernel-client/wasm/sse_wasm.js");
        await mod.default("/packages/kernel-client/wasm/sse_wasm_bg.wasm");
        const packs = await Promise.all(packsAt.map(async (p) => (await fetch(`/${p}`)).json()));
        // The same function as in Node, sent as text so both runners time the same code.
        const READS = (0, eval)(`(${src.reads})`);
        const run = (0, eval)(`(${src.run})`);
        return run(mod.Kernel, () => performance.now(), packs, READS, n);
      },
      { src: { reads: readsSource(), run: runSource() }, packsAt: PACKS, runs },
    );
    return { runner: `Chromium ${browser.version()}${rate > 1 ? `, CPU slowed ${rate}×` : ""}`, results };
  } finally {
    await browser.close();
    server.close();
  }
}

// The reads and the loop as text for the page (functions do not cross into it any other way).
function readsSource() {
  return `[${READS.map((r) => `{ id: ${JSON.stringify(r.id)}, rung: ${JSON.stringify(r.rung)}, what: ${JSON.stringify(r.what)}, ${r.ask ? `ask: ${JSON.stringify(r.ask)}` : `first: ${JSON.stringify(r.first)}, then: ${r.then.toString()}`} }`).join(", ")}]`;
}
function runSource() {
  return `function (Kernel, now, packs, READS, runs) { const UNIVERSE = ${JSON.stringify(UNIVERSE)}; return (${runAll.toString()})(Kernel, now, packs); }`;
}

const all = [];
for (const r of runners) {
  const t0 = performance.now();
  const rep = r === "node" ? await inNode() : r === "chromium" ? await inChromium(1) : r === "chromium-cpu4" ? await inChromium(4) : null;
  if (!rep) throw new Error(`unknown runner ${r}`);
  all.push(rep);
  console.log(`\n${rep.runner} (${((performance.now() - t0) / 1000).toFixed(0)} s)`);
  for (const x of rep.results) console.log(`  ${x.id.padEnd(22)} cold ${fmt(x.cold_ms).padStart(8)}  warm ${fmt(x.warm_ms).padStart(8)}  setup ${fmt(x.setup_ms).padStart(8)}  ${String(x.reply_kb).padStart(6)} kB${x.error ? `  ERROR ${x.error}` : ""}`);
}
function fmt(x) {
  return x === null ? "-" : x >= 1000 ? `${(x / 1000).toFixed(2)} s` : `${x.toFixed(0)} ms`;
}
const c = cpus();
const report = { schema: "sse.gz0-timings/1", made_on: new Date().toISOString().slice(0, 10), machine: { cpu: c[0]?.model, cores: c.length }, runs, packs: PACKS, universe: UNIVERSE, runners: all };
if (out) writeFileSync(out, JSON.stringify(report, null, 1) + "\n");
