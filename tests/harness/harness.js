import init, { harness_run } from "./pkg/sse_wasm.js";

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
  const kpass = await kernelCheck();
  document.body.dataset.result = pass && kpass ? "pass" : "fail";
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
  $("cap").textContent = `This device: ${Math.round(cap.unitsPerSecond).toLocaleString()} work units per second, tier "${cap.tier}" (runs up to ${cap.runsUpTo} work locally). Speed is shown only; it never enters a hash.`;
  worker.terminate();
  return ok;
}
main().catch((e) => { $("verdict").textContent = "Error: " + e; $("verdict").className = "verdict bad"; document.body.dataset.result = "error"; });
