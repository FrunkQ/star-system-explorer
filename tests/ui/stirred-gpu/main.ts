// The stirred face's GPU carrying (stirred-gpu.ts) against its processor twin (stirred-carry.ts `sourcePlaces`), on
// the kernel's own tables (tests/fixtures/weather/carry.json). Built and run by tests/ui/stirred-gpu.e2e.mjs.
import { WebGPURenderer } from "three/webgpu";
import { Carrier, frame, generations, latLon, sourcePlaces, unit, patchPlace, type CarryTable } from "../../../packages/renderer/src/stirred-carry.ts";
import { carryOnGpu, packCarry } from "../../../packages/renderer/src/stirred-gpu.ts";
import fixture from "../../fixtures/weather/carry.json";

type Out = { backend?: string; cells?: { k: number; gpu: number[]; cpu: number[]; km: number[] }[]; ms?: number; error?: string };
const w = window as unknown as { __done?: Out };
const km = (a: number, la1: number, lo1: number, la2: number, lo2: number) => {
  const p = unit(la1, lo1);
  const q = unit(la2, lo2);
  return (Math.acos(Math.min(1, p[0] * q[0] + p[1] * q[1] + p[2] * q[2])) * a) / 1000;
};

(async () => {
  const out: Out = {};
  try {
    const r = new WebGPURenderer({ canvas: document.getElementById("c") as HTMLCanvasElement, forceWebGL: location.search.includes("gl") });
    await r.init();
    out.backend = (r.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? "webgpu" : "webgl2";
    const t = (fixture as unknown as { neptune: { table: CarryTable } }).neptune.table;
    // An 8 × 8 patch round the kernel's patch: 64 cells, as the design's check asks.
    const n = 8;
    const places = new Float32Array(n * n * 2);
    for (let j = 0; j < n; j++) for (let i = 0; i < n; i++) places.set(patchPlace(t.radius_m, -30, 40, 8000, n, i, j), (j * n + i) * 2);
    const packed = packCarry(t);
    if (location.search.includes("probe")) {
      // A look inside, for when the check fails: the places and ages, the winds, one step back, against the processor's.
      const c = new Carrier(t);
      const look = async (kind: 0 | 1 | 2, h = 0) => {
        const t1 = performance.now();
        const got = await carryOnGpu(r, packed, places, n, n, { probe: { kind, h } });
        console.log(`look ${kind}: ${((performance.now() - t1) / 1000).toFixed(1)} s`);
        return got;
      };
      const [a, b, s] = [await look(0), await look(1), await look(2, t.dt_s)];
      const gens = (k: number) => generations(t, places[k * 2]!);
      for (const k of [0, 9, 36, 63]) {
        const la = places[k * 2]!;
        const lo = places[k * 2 + 1]!;
        const v = c.wind(unit(la, lo), t.day);
        const [e, nn] = frame(la, lo);
        const q = latLon(c.step(unit(la, lo), t.day, -t.dt_s));
        console.log(`cell ${k}: GPU place ${Array.from(a.subarray(k * 4, k * 4 + 4)).map((x) => x.toFixed(3))} CPU ${la.toFixed(3)},${lo.toFixed(3)} ages ${gens(k).map((g) => g[0].toFixed(3))}`);
        console.log(`  wind GPU ${Array.from(b.subarray(k * 4, k * 4 + 4)).map((x) => x.toFixed(3))} CPU ${(v[0] * e[0] + v[1] * e[1] + v[2] * e[2]).toFixed(3)},${(v[0] * nn[0] + v[1] * nn[1] + v[2] * nn[2]).toFixed(3)}`);
        console.log(`  step GPU ${Array.from(s.subarray(k * 4, k * 4 + 4)).map((x) => x.toFixed(4))} CPU ${q.map((x) => x.toFixed(4))}`);
      }
      w.__done = { backend: out.backend, cells: [], ms: 0 };
      return;
    }
    const t0 = performance.now();
    let draws = 0;
    const gpu = await carryOnGpu(r, packed, places, n, n, {
      chunk: 1,
      each: async () => {
        draws++;
        if (draws % 200 === 0) console.log(`draw ${draws} at ${((performance.now() - t0) / 1000).toFixed(1)} s`);
      },
    });
    out.ms = performance.now() - t0;
    const c = new Carrier(t);
    out.cells = [];
    for (let k = 0; k < n * n; k++) {
      const s = sourcePlaces(c, places[k * 2]!, places[k * 2 + 1]!);
      const g = Array.from(gpu.subarray(k * 4, k * 4 + 4));
      const cpu = [s.places[0][0], s.places[0][1], s.places[1][0], s.places[1][1]];
      out.cells.push({ k, gpu: g, cpu, km: [km(t.radius_m, g[0]!, g[1]!, cpu[0]!, cpu[1]!), km(t.radius_m, g[2]!, g[3]!, cpu[2]!, cpu[3]!)] });
    }
  } catch (e) {
    out.error = String((e as Error).stack ?? e);
  }
  w.__done = out;
})();
