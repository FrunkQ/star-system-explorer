import init, { harness_run, engines_run, engine_names, engine_run } from "./pkg/sse_wasm.js";
import { benchReport, runBench } from "./bench.js";

const $ = (id) => document.getElementById(id);
$("ua").textContent = navigator.userAgent;

async function main() {
  await init();
  const golden = await (await fetch("./pkg/golden.json", { cache: "no-store" })).json();
  const t0 = performance.now(); // display only; never enters a hash
  const report = JSON.parse(harness_run());
  const ms = performance.now() - t0;
  const pass = report.overall === golden.overall && report.encoding_failures === 0;
  const v = $("verdict");
  v.textContent = pass ? "PASS: identical to the native golden hashes" : "FAIL: this device differs from the golden hashes";
  v.className = "verdict " + (pass ? "ok" : "bad");
  $("overall").textContent = report.overall;
  $("timing").textContent = `Took ${Math.round(ms)} ms.`;
  const rows = Object.entries(report.digests).map(([k, d]) => {
    const same = golden.digests[k] === d;
    return `<tr><td>${same ? "✓" : '<span class="x">✗</span>'} ${k}</td><td class="mono">${d}</td></tr>`;
  });
  $("table").innerHTML = rows.join("");
  $("copy").onclick = () => navigator.clipboard?.writeText(
    `SSE4 harness ${report.version}: ${pass ? "PASS" : "FAIL"}\noverall ${report.overall}\n${$("kverdict").textContent}\n${navigator.userAgent}`);
  document.body.dataset.overall = report.overall;
  const epass = await enginesCheck();
  const kpass = await kernelCheck();
  document.body.dataset.result = pass && epass && kpass ? "pass" : "fail";
  benchSection(report.version);
}

// PF2: the bench. Only on request (a button, or ?bench in the address for the bench script): it takes a while.
let capability = null;
let kernelVersion = null;
function benchSection(version) {
  const sec = $("bench");
  sec.hidden = false;
  let rep = null;
  const ms = (x) => (x >= 1000 ? `${(x / 1000).toFixed(1)} s` : `${x >= 100 ? Math.round(x) : x.toFixed(1)} ms`);
  const runs = Number(new URLSearchParams(location.search).get("runs") ?? 3) || 3;
  const go = async () => {
    $("benchgo").disabled = true;
    const doc = await (await fetch("./pkg/perf-chunks.json", { cache: "no-store" })).json();
    const results = await runBench({ engine_names, engine_run }, doc, {
      runs,
      onProgress: (d, t, name) => ($("benchnote").textContent = name ? `${d} of ${t}: ${name}…` : "Done."),
      yieldTo: () => new Promise((r) => setTimeout(r, 0)),
    });
    rep = benchReport(results, doc, { device: await describe(), kernel: kernelVersion ?? undefined, site: location.host, runs });
    rep.app.harness = version;
    $("benchtable").innerHTML = rep.chunks
      .map((c) => `<tr><td><strong>${c.name}</strong></td><td class="n">${ms(c.median_ms)}</td></tr>` + c.sources.map((s) => `<tr><td class="muted">&nbsp;&nbsp;${s.source.slice(6)}${s.unsteady ? " (unsteady)" : ""}</td><td class="n">${ms(s.median_ms)}</td></tr>`).join(""))
      .join("");
    const shaky = rep.chunks.flatMap((c) => c.sources.filter((x) => x.unsteady).map((x) => x.source.slice(6)));
    if (shaky.length) $("benchnote").textContent = `Done, but one run of ${shaky.join(", ")} took over twice its usual time: the device may have paused (screen dimmed or another app). The medians are still fair; for a clean report keep the screen on and run again.`;
    $("benchacts").hidden = false;
    $("benchgo").disabled = false;
    window.__benchReport = rep;
    document.body.dataset.bench = "done";
  };
  const text = () => JSON.stringify({ ...rep, label: $("benchlabel").value.trim() }, null, 2);
  $("benchgo").onclick = go;
  $("benchcopy").onclick = async () => {
    try { await navigator.clipboard.writeText(text()); $("benchnote").textContent = "Copied; nothing was sent."; } catch { $("benchnote").textContent = "This browser would not copy; use Save."; }
  };
  $("benchsave").onclick = () => {
    const a = document.createElement("a");
    a.href = URL.createObjectURL(new Blob([text()], { type: "application/json" }));
    a.download = `sse4-bench-${($("benchlabel").value.trim() || "device").replace(/[^A-Za-z0-9-]+/g, "-").toLowerCase()}.json`;
    a.click();
  };
  if (new URLSearchParams(location.search).has("bench")) go();
}

// What this device says about itself (the readout's describeDevice, packages/ui-contract/src/perf.ts, in plain JS).
async function describe() {
  let gpu = { backend: "none" };
  try {
    const a = await navigator.gpu?.requestAdapter();
    if (a) { const i = a.info ?? {}; gpu = { backend: "webgpu", adapter: [i.vendor, i.architecture, i.device, i.description].filter(Boolean).join(" ") || undefined }; }
  } catch {}
  if (gpu.backend === "none") {
    try {
      const gl = document.createElement("canvas").getContext("webgl2");
      if (gl) { const x = gl.getExtension("WEBGL_debug_renderer_info"); gpu = { backend: "webgl2", adapter: x ? String(gl.getParameter(x.UNMASKED_RENDERER_WEBGL)) : undefined }; }
    } catch {}
  }
  return {
    user_agent: navigator.userAgent,
    ...(navigator.hardwareConcurrency ? { cores: navigator.hardwareConcurrency } : {}),
    ...(navigator.deviceMemory ? { memory_gb: navigator.deviceMemory } : {}),
    screen: `${screen.width}×${screen.height}`,
    pixel_ratio: devicePixelRatio,
    ...(capability ? { tier: capability.tier, units_per_second: Math.round(capability.unitsPerSecond) } : {}),
    gpu,
    reduced_motion: matchMedia("(prefers-reduced-motion: reduce)").matches,
  };
}

// K7 to K10 and G0: resolve, sift, observe, suite and cosmos scenarios against native digests.
async function enginesCheck() {
  const golden = await (await fetch("./pkg/engines.json", { cache: "no-store" })).json();
  const got = JSON.parse(engines_run());
  const keys = Object.keys(golden);
  const bad = keys.filter((k) => JSON.stringify(got[k]) !== JSON.stringify(golden[k]));
  const v = $("everdict");
  if (v) {
    v.textContent = bad.length ? `FAIL: engine scenarios differ: ${bad.join(", ")}` : `PASS: engine scenarios (${keys.join(", ")}) identical to native`;
    v.className = "verdict " + (bad.length ? "bad" : "ok");
  }
  return bad.length === 0;
}

async function kernelCheck() {
  const { KernelClient, browserPort, runContract } = await import("./pkg/client/index.js");
  const get = async (p, as = "json") => { const r = await fetch(p, { cache: "no-store" }); return as === "json" ? r.json() : r.text(); };
  const names = await get("./pkg/fixtures/packs.json");
  const packs = {};
  for (const n of names) packs[n] = await get(`./pkg/fixtures/packs/${n}.json`);
  const requests = await get("./pkg/fixtures/requests.json");
  const golden = (await get("./pkg/contract.jsonl", "text")).trimEnd().split("\n");
  const worker = new Worker(new URL("./pkg/client/worker.js", import.meta.url), { type: "module" });
  const client = new KernelClient(browserPort(worker), { jsUrl: new URL("./pkg/sse_wasm.js", import.meta.url).href });
  const got = await runContract(client, requests, packs);
  const bad = got.map((l, i) => (l === golden[i] ? -1 : i)).filter((i) => i >= 0);
  const ok = bad.length === 0 && got.length === golden.length;
  const v = $("kverdict");
  v.textContent = ok
    ? `PASS: kernel API through a Web Worker, ${got.length} of ${golden.length} responses identical to native`
    : `FAIL: kernel API responses differ at ${bad.join(", ") || "length"}`;
  v.className = "verdict " + (ok ? "ok" : "bad");
  const cap = await client.measure(20000);
  capability = cap;
  kernelVersion = (await client.info()).kernel_version;
  $("cap").textContent = `This device: ${Math.round(cap.unitsPerSecond).toLocaleString()} work units per second, tier "${cap.tier}" (runs up to ${cap.runsUpTo} work locally). Speed is shown only; it never enters a hash.`;
  worker.terminate();
  return ok;
}
main().catch((e) => { $("verdict").textContent = "Error: " + e; $("verdict").className = "verdict bad"; document.body.dataset.result = "error"; });
