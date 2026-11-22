import { readFileSync } from "node:fs";
const bytes = readFileSync(new URL("./target/wasm32-unknown-unknown/release/gh0bench.wasm", import.meta.url));
const { instance } = await WebAssembly.instantiate(bytes, {});
const e = instance.exports;
const t = (f) => { const a = performance.now(); const r = f(); return [performance.now() - a, r]; };
for (const [label, f] of [
  ["setup (cosmology)", () => e.setup()],
  ["tree chart (1e-3, counts)", () => e.tree(1e-3, 0)],
  ["tree chart + G3 for all ~55", () => e.tree(1e-3, 1000)],
  ["tree survey (1e-5, counts)", () => e.tree(1e-5, 0)],
  ["orbits 55 x 14000", () => e.orbits(55, 14000)],
  ["orbits 2000 x 14000", () => e.orbits(2000, 14000)],
]) { const [ms, r] = t(f); console.log(`${label}: ${ms.toFixed(1)} ms (result ${r.toFixed(1)})`); }
