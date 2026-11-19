// Run the wasm harness in Node and compare with the golden file. Exit 1 on mismatch.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
const here = dirname(fileURLToPath(import.meta.url));
const { initSync, harness_run, engines_run } = await import(join(here, "pkg/sse_wasm.js"));
initSync({ module: readFileSync(join(here, "pkg/sse_wasm_bg.wasm")) });
const r = JSON.parse(harness_run());
const g = JSON.parse(readFileSync(join(here, "../golden/harness.json"), "utf8"));
const ok = r.overall === g.overall && r.encoding_failures === 0;
console.log(`node ${process.version}: ${ok ? "PASS" : "FAIL"} overall ${r.overall}`);
// K7 to K10 engine scenarios against their native golden files.
const e = JSON.parse(engines_run());
const bad = ["resolve", "sift", "observe", "suite", "cosmos", "formation", "formation-early", "formation-accrete", "formation-accrete-dole", "formation-growth", "formation-upheaval", "formation-leftovers", "formation-pipeline", "classify", "life", "place", "time", "time-generated", "surface", "surface-atlas", "surface-plates", "surface-wear", "surface-impacts", "surface-regimes", "surface-cover", "cosmology", "haloes", "galaxy", "envelope", "life-budget", "life-history", "life-biosphere", "stars", "life-signatures", "life-feedback", "nav"].filter(
  (k) => JSON.stringify(e[k]) !== JSON.stringify(JSON.parse(readFileSync(join(here, `../golden/${k}.json`), "utf8"))));
console.log(`node engines: ${bad.length ? "FAIL " + bad.join(", ") : "PASS resolve, sift, observe, suite, cosmos, formation, formation-early, formation-accrete, formation-accrete-dole, formation-growth, formation-upheaval, formation-leftovers, formation-pipeline, classify, life, place, time, time-generated, surface, surface-atlas, surface-plates, surface-wear, surface-impacts, surface-regimes, surface-cover, cosmology, haloes, galaxy, envelope, life-budget, life-history, life-biosphere, stars, life-signatures, life-feedback, nav"}`);
process.exit(ok && !bad.length ? 0 : 1);
