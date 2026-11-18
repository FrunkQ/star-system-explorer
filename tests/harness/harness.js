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
    `SSE4 harness ${report.version}: ${pass ? "PASS" : "FAIL"}\noverall ${report.overall}\n${navigator.userAgent}`);
  document.body.dataset.result = pass ? "pass" : "fail";
  document.body.dataset.overall = report.overall;
}
main().catch((e) => { $("verdict").textContent = "Error: " + e; $("verdict").className = "verdict bad"; document.body.dataset.result = "error"; });
