// VP4 acceptance: placement by measured speed (SSE4-visual-polish-design.md 8.1 to 8.3). A device is placed at the
// richest tier whose standard frame is predicted within 60% of its budget; WebGL2 can earn High by speed; Ultra needs
// WebGPU with compute and headroom; the person's choice overrides and says so; a hot device is lowered for the visit;
// the probe's cache is per adapter and browser.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { factorsFrom, gpuTiersAllowed, HeatWatch, HEAT_WINDOW_MS, loadProbe, loadQuality, place, PLACE_MARGIN, PROBE_SCENES, probeKey, saveProbe, saveQuality, drawingBench, BENCH_PIXELS, type Capabilities, type ProbeResult } from "../src/probe.ts";
import { estimateMs, passFactor, plan, TIER_ORDER, type RecipeDoc } from "../src/recipe.ts";
import { LIBRARY } from "../src/passes.ts";
import { passShares } from "../src/runtime.ts";

const trueColour: RecipeDoc = JSON.parse(readFileSync(new URL("../src/recipes/true-colour.json", import.meta.url), "utf8"));
const PIXELS = 640 * 640;

const probe = (f: number, g = f, m = f): Pick<ProbeResult, "factors"> => ({ factors: { fill: f, march: m, geometry: g } });
const webgpu = { backend: "webgpu" as const, compute: true };
const webgl2 = { backend: "webgl2" as const, compute: false };

/** A recipe declaring Ultra, made from true colour with a heavier air on Ultra. */
const withUltra: RecipeDoc = {
  ...trueColour,
  id: "true-colour-ultra-test",
  tiers: ["still", "gpu-ultra", "gpu-high", "gpu-basic", "canvas", "text"],
  passes: [...trueColour.passes.map((p) => (p.id === "relief" ? { ...p, tiers: [...(p.tiers ?? []), "gpu-ultra" as const] } : p)), { id: "air-ultra", pass: "atmosphere", params: { steps: 64, light_steps: 16 }, tiers: ["gpu-ultra"] }],
};
const ultraLibrary = LIBRARY.map((p) => (p.id === "relief-globe" || p.id === "atmosphere" ? { ...p, tiers: [...p.tiers, "gpu-ultra" as const], cost: { ...p.cost, "gpu-ultra": p.cost["gpu-high"] } } : p));

test("gpu-ultra sits between still and gpu-high", () => {
  assert.deepEqual(TIER_ORDER, ["still", "gpu-ultra", "gpu-high", "gpu-basic", "canvas", "text"]);
});

test("a pass's device factor follows its own load mix", () => {
  const f = { fill: 1, march: 4, geometry: 2 };
  assert.equal(passFactor({ fixed_ms: 0, per_mpix_ms: 1 }, f), 1, "no mix: bound by fill");
  assert.equal(passFactor({ fixed_ms: 0, per_mpix_ms: 1, load: { march: 1 } }, f), 4);
  assert.equal(passFactor({ fixed_ms: 0, per_mpix_ms: 1, load: { geometry: 1, fill: 1 } }, f), 1.5);
  // estimateMs with factors scales GPU tiers only.
  const p = plan(trueColour, LIBRARY, "gpu-high");
  const ref = estimateMs(p, LIBRARY, PIXELS);
  assert.ok(Math.abs(estimateMs(p, LIBRARY, PIXELS, {}, { fill: 2, march: 2, geometry: 2 }) - 2 * ref) < 1e-9);
  const c = plan(trueColour, LIBRARY, "canvas");
  assert.equal(estimateMs(c, LIBRARY, PIXELS, {}, { fill: 9, march: 9, geometry: 9 }), estimateMs(c, LIBRARY, PIXELS));
});

test("factors are measured over reference, floored against a coarse timer", () => {
  const r = factorsFrom({ fill: PROBE_SCENES.fill.reference_ms * 2, march: PROBE_SCENES.march.reference_ms, geometry: 0 });
  assert.equal(r.fill, 2);
  assert.equal(r.march, 1);
  assert.equal(r.geometry, 0.05);
  assert.equal(factorsFrom({ fill: NaN, march: 1, geometry: 1 }).fill, Infinity);
});

test("no GPU: the 2D canvas, said plainly", () => {
  const p = place({ caps: { backend: null, compute: false }, probe: null, pixels: PIXELS, recipe: trueColour, library: LIBRARY });
  assert.equal(p.tier, "canvas");
  assert.match(p.note, /2D canvas/);
});

test("before the probe returns, every GPU device starts at Light", () => {
  for (const caps of [webgpu, webgl2]) {
    const p = place({ caps, probe: null, pixels: PIXELS, recipe: trueColour, library: LIBRARY });
    assert.equal(p.tier, "gpu-basic");
    assert.equal(p.by, "capabilities");
    assert.match(p.note, /measured/);
  }
});

test("a fast WebGL2 device earns High by its measured speed (question 3)", () => {
  const p = place({ caps: webgl2, probe: probe(0.5), pixels: PIXELS, recipe: trueColour, library: LIBRARY });
  assert.equal(p.earned, "gpu-high");
  assert.equal(p.tier, "gpu-high");
  assert.match(p.note, /measured speed on WebGL2/);
  assert.ok(!gpuTiersAllowed(webgl2).includes("gpu-ultra"), "WebGL2 has no compute, so never Ultra");
});

test("a slow WebGPU device is placed at Light, not High", () => {
  const p = place({ caps: webgpu, probe: probe(6), pixels: PIXELS, recipe: trueColour, library: LIBRARY });
  assert.equal(p.tier, "gpu-basic");
  const high = p.predictions.find((x) => x.tier === "gpu-high")!;
  assert.ok(high.ms! > high.budget_ms * PLACE_MARGIN);
});

test("the 60% margin is the line", () => {
  const ref = estimateMs(plan(trueColour, LIBRARY, "gpu-high"), LIBRARY, PIXELS);
  const at = (16.7 * PLACE_MARGIN) / ref;
  assert.equal(place({ caps: webgl2, probe: probe(at * 0.99), pixels: PIXELS, recipe: trueColour, library: LIBRARY }).tier, "gpu-high");
  assert.equal(place({ caps: webgl2, probe: probe(at * 1.01), pixels: PIXELS, recipe: trueColour, library: LIBRARY }).tier, "gpu-basic");
});

test("Ultra: earned by headroom until a recipe declares it, then by that recipe's own prediction", () => {
  const fast = place({ caps: webgpu, probe: probe(0.2), pixels: PIXELS, recipe: trueColour, library: LIBRARY });
  assert.equal(fast.earned, "gpu-ultra");
  assert.equal(fast.tier, "gpu-high", "the recipe has no Ultra form, so the view draws at High");
  const mid = place({ caps: webgpu, probe: probe(0.9), pixels: PIXELS, recipe: trueColour, library: LIBRARY });
  assert.equal(mid.earned, "gpu-high");
  const u = place({ caps: webgpu, probe: probe(0.2), pixels: PIXELS, recipe: withUltra, library: ultraLibrary });
  assert.equal(u.tier, "gpu-ultra");
  // Slow at marching only: the heavy air on Ultra does not fit, though High does.
  const marchSlow = place({ caps: webgpu, probe: { factors: { fill: 0.3, geometry: 0.3, march: 1 } }, pixels: PIXELS, recipe: withUltra, library: ultraLibrary });
  assert.equal(marchSlow.tier, "gpu-high");
  // WebGPU without compute never gets Ultra.
  assert.notEqual(place({ caps: { backend: "webgpu", compute: false }, probe: probe(0.1), pixels: PIXELS, recipe: withUltra, library: ultraLibrary }).tier, "gpu-ultra");
});

test("the person's choice overrides the probe, and the note says it is theirs", () => {
  const light = place({ caps: webgpu, probe: probe(0.2), pixels: PIXELS, recipe: trueColour, library: LIBRARY, choice: "light" });
  assert.equal(light.tier, "gpu-basic");
  assert.equal(light.by, "choice");
  assert.match(light.note, /by you/);
  const full = place({ caps: webgl2, probe: probe(8), pixels: PIXELS, recipe: trueColour, library: LIBRARY, choice: "full" });
  assert.equal(full.tier, "gpu-high");
  assert.match(full.note, /above what this device measured/);
  const ultra = place({ caps: webgl2, probe: probe(0.2), pixels: PIXELS, recipe: trueColour, library: LIBRARY, choice: "ultra" });
  assert.equal(ultra.tier, "gpu-high", "Ultra asked on WebGL2: the richest it allows");
});

test("the heat watch: three steps down inside a minute lowers the ceiling once", () => {
  const h = new HeatWatch();
  assert.equal(h.down(0, "gpu-high"), null);
  assert.equal(h.down(20_000, "gpu-high"), null);
  assert.equal(h.down(40_000, "gpu-high"), "gpu-basic");
  assert.equal(h.ceiling, "gpu-basic");
  // Spread-out steps do not count.
  const k = new HeatWatch();
  for (let i = 0; i < 6; i++) assert.equal(k.down(i * HEAT_WINDOW_MS, "gpu-high"), null);
  // Already at Light: nowhere lower on the GPU.
  const b = new HeatWatch();
  b.down(0, "gpu-basic");
  b.down(1, "gpu-basic");
  assert.equal(b.down(2, "gpu-basic"), null);
  // And placement honours the lowered ceiling.
  const p = place({ caps: webgl2, probe: probe(0.5), pixels: PIXELS, recipe: trueColour, library: LIBRARY, heat_ceiling: h.ceiling });
  assert.equal(p.tier, "gpu-basic");
  assert.equal(p.earned, "gpu-high");
  assert.match(p.note, /hot device/);
});

function memStore() {
  const m = new Map<string, string>();
  return { getItem: (k: string) => m.get(k) ?? null, setItem: (k: string, v: string) => void m.set(k, v), removeItem: (k: string) => void m.delete(k) };
}

test("the probe is cached per adapter and browser, and a new one measures again", () => {
  const s = memStore();
  const caps = { backend: "webgpu" as const, adapter: "acme gpu" };
  const key = probeKey(caps, "Browser/1");
  const r: ProbeResult = { schema: "sse.render-probe/1", key, backend: "webgpu", method: "timestamp", scenes_ms: { fill: 0.4, march: 1.2, geometry: 0.5 }, factors: { fill: 1, march: 1, geometry: 1 }, steady: true };
  saveProbe(s, r);
  assert.deepEqual(loadProbe(s, key), r);
  assert.equal(loadProbe(s, probeKey(caps, "Browser/2")), null, "browser updated");
  assert.equal(loadProbe(s, probeKey({ ...caps, adapter: "other gpu" }, "Browser/1")), null, "adapter changed");
  s.setItem("sse4.render.probe", "{not json");
  assert.equal(loadProbe(s, key), null);
  assert.equal(loadProbe(null, key), null);
});

test("the quality choice is kept, Auto by default", () => {
  const s = memStore();
  assert.equal(loadQuality(s), "auto");
  saveQuality(s, "ultra");
  assert.equal(loadQuality(s), "ultra");
  s.setItem("sse4.render.quality", "silly");
  assert.equal(loadQuality(s), "auto");
});

test("per-pass shares of a frame sum to one", () => {
  const p = plan(trueColour, LIBRARY, "gpu-high");
  const sh = passShares(p, LIBRARY, PIXELS, probe(1, 1, 3));
  assert.ok(sh.length >= 2);
  assert.ok(Math.abs(sh.reduce((a, x) => a + x.share, 0) - 1) < 1e-9);
  // Slow marching gives the air a bigger share.
  const air = (f: number) => passShares(p, LIBRARY, PIXELS, probe(1, 1, f)).find((x) => x.pass === "atmosphere")!.share;
  assert.ok(air(3) > air(1));
});

test("the bench's drawing row places at 1080p", () => {
  const caps: Capabilities = { backend: "webgl2", compute: false, half_float: true, textures_3d: true, max_texture: 4096, timestamps: false, memory_gb: null, screen_px: 0, pixel_ratio: 1, reduced_motion: false, save_data: false, adapter: "x" };
  const r: ProbeResult = { schema: "sse.render-probe/1", key: "k", backend: "webgl2", method: "readback", scenes_ms: { fill: 4, march: 12, geometry: 5 }, factors: { fill: 10, march: 10, geometry: 10 }, steady: true };
  const row = drawingBench(caps, r, trueColour, LIBRARY);
  assert.equal(row.earned, "gpu-basic");
  assert.equal(row.predictions.length, 2);
  const high = row.predictions.find((x) => x.tier === "gpu-high")!;
  assert.ok(Math.abs(high.ms! - estimateMs(plan(trueColour, LIBRARY, "gpu-high"), LIBRARY, BENCH_PIXELS, {}, r.factors)) < 1e-9);
});
