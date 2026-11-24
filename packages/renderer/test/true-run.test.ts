// VP4 part 2: one controller per True view. Placed by the device, stepped down by the governor on slow frames, the
// heat watch lowers the visit's ceiling, the person's choice re-places at once, and at rest it hands the still passes.
import { test } from "node:test";
import assert from "node:assert/strict";
import { TrueRun, type TrueDevice } from "../src/true-run.ts";
import { readFileSync } from "node:fs";
import { LIBRARY } from "../src/passes.ts";
import { checkRecipe, estimateMs, plan, type RecipeDoc } from "../src/recipe.ts";
import type { Tier } from "../../ui-contract/src/index.ts";
import { HeatWatch, type Capabilities, type ProbeResult, type QualityChoice } from "../src/probe.ts";

const SYSTEM_TRUE: RecipeDoc = JSON.parse(readFileSync(new URL("../src/recipes/system-true.json", import.meta.url), "utf8"));
const PIXELS = 1600 * 900;
const caps: Capabilities = { backend: "webgl2", compute: false, half_float: true, textures_3d: true, max_texture: 8192, timestamps: false, memory_gb: 8, screen_px: PIXELS, pixel_ratio: 1, reduced_motion: false, save_data: false, adapter: "test" };

function stub(f: number, q: QualityChoice = "auto") {
  const heat = new HeatWatch();
  const listeners = new Set<() => void>();
  const d: TrueDevice & { q: QualityChoice; fire(): void } = {
    q,
    caps: async () => caps,
    speed: async (): Promise<ProbeResult> => ({ schema: "sse.render-probe/1", key: "k", backend: "webgl2", method: "readback", scenes_ms: { fill: 1, march: 1, geometry: 1 }, factors: { fill: f, march: f, geometry: f }, steady: true }),
    quality: () => d.q,
    heatCeiling: () => heat.ceiling,
    heatDown: (now: number, t: Tier) => heat.down(now, t),
    onChange: (fn) => (listeners.add(fn), () => listeners.delete(fn)),
    fire: () => listeners.forEach((l) => l()),
  };
  return d;
}

test("the system True recipe checks against the pass library", () => {
  assert.deepEqual(checkRecipe(SYSTEM_TRUE, LIBRARY).errors, []);
  // High has the full glare ladder at half size; Light three rungs at quarter size (the finish's tier knob).
  const h = plan(SYSTEM_TRUE, LIBRARY, "gpu-high").passes.find((p) => p.pass === "true-finish")!.params;
  const b = plan(SYSTEM_TRUE, LIBRARY, "gpu-basic").passes.find((p) => p.pass === "true-finish")!.params;
  assert.deepEqual([h.glare_rungs, h.glare_scale, b.glare_rungs, b.glare_scale], [6, 0.5, 3, 0.25]);
});

test("placed by speed: a quick WebGL2 device at High, a slow one at Light", async () => {
  const fast = new TrueRun({ recipe: SYSTEM_TRUE, library: LIBRARY, pixels: PIXELS, backend: "webgl2", source: "render.system", device: stub(0.5) });
  await fast.place();
  assert.equal(fast.settings().tier, "gpu-high");
  assert.equal(fast.settings().params.finish!.glare_rungs, 6);
  const slow = new TrueRun({ recipe: SYSTEM_TRUE, library: LIBRARY, pixels: PIXELS, backend: "webgl2", source: "render.system", device: stub(4) });
  await slow.place();
  assert.equal(slow.settings().tier, "gpu-basic");
  assert.match(slow.status().note, /measured speed/);
});

test("slow frames walk the degrade order; the badge says what was reduced", async () => {
  const r = new TrueRun({ recipe: SYSTEM_TRUE, library: LIBRARY, pixels: PIXELS, backend: "webgl2", source: "render.system", device: stub(0.5) });
  await r.place();
  let changed = 0;
  for (let i = 0; i < 40; i++) if (r.frame(40, i * 40)) changed++;
  assert.ok(changed >= 1);
  assert.ok(r.settings().render_scale < 1 || r.status().level > 0);
  assert.equal(r.status().badge.reduced, true);
});

test("a hot device: steps down three times in a minute, the visit's ceiling drops", async () => {
  const d = stub(0.5);
  const r = new TrueRun({ recipe: SYSTEM_TRUE, library: LIBRARY, pixels: PIXELS, backend: "webgl2", source: "render.system", device: d });
  await r.place();
  let t = 0;
  for (let i = 0; i < 200 && d.heatCeiling() === null; i++) r.frame(60, (t += 60));
  assert.equal(d.heatCeiling(), "gpu-basic");
  await new Promise((res) => setTimeout(res, 0));
  await r.place();
  assert.equal(r.settings().tier, "gpu-basic");
});

test("the person's choice re-places at once", async () => {
  const d = stub(0.5);
  let told = 0;
  const r = new TrueRun({ recipe: SYSTEM_TRUE, library: LIBRARY, pixels: PIXELS, backend: "webgl2", source: "render.system", device: d, onChange: () => told++ });
  await r.place();
  d.q = "light";
  d.fire();
  await new Promise((res) => setTimeout(res, 0));
  assert.equal(r.settings().tier, "gpu-basic");
  assert.match(r.status().note, /by you/);
  assert.ok(told >= 2);
  r.dispose();
});

test("at rest: the still passes at full size, and the rest clock aims by tier", async () => {
  const r = new TrueRun({ recipe: SYSTEM_TRUE, library: LIBRARY, pixels: PIXELS, backend: "webgl2", source: "render.system", device: stub(4) });
  await r.place();
  const s = r.settings(true);
  assert.equal(s.render_scale, 1);
  assert.equal(s.params.bodies!.sphere_detail, 128);
  assert.equal(s.params.finish!.glare_scale, 1);
  r.frame(20, 1000);
  assert.equal(r.rest.state, "waiting");
  assert.ok(r.rest.next(1600));
});

test("a forced tier stays, and says so", async () => {
  const r = new TrueRun({ recipe: SYSTEM_TRUE, library: LIBRARY, pixels: PIXELS, backend: "webgl2", tier: "gpu-high", source: "render.system", device: stub(9) });
  await r.place();
  assert.equal(r.settings().tier, "gpu-high");
  assert.match(r.status().note, /as set/);
  assert.ok(estimateMs(r.plan, LIBRARY, PIXELS) > 0);
});
