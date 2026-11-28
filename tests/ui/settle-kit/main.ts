// The settle kit on the GPU (settle-kit.ts) against its processor twins (settle-kit-cpu.ts), on a 64-wide patch.
// Built and run by tests/ui/settle-kit.e2e.mjs.
import { WebGPURenderer } from "three/webgpu";
import { benchKit, drawKit } from "../../../packages/renderer/src/settle-kit.ts";
import { carryCpu, exposureCpu, fadeCpu, rowMeans } from "../../../packages/renderer/src/settle-kit-cpu.ts";
import { windTable } from "../../../packages/renderer/src/wind-table.ts";

const W = 64;
const H = 32;
const DAY = 86_400;
const picture = (f: (i: number, j: number) => [number, number, number]): Float32Array => {
  const out = new Float32Array(W * H * 4);
  for (let j = 0; j < H; j++) for (let i = 0; i < W; i++) out.set([...f(i, j), 255], (j * W + i) * 4);
  return out;
};
// Smooth east-west, so a place off by a fraction of a pixel shows as that fraction of the steepest change (STEEP a pixel).
const bands = picture((i, j) => [128 + 100 * Math.cos((2 * Math.PI * i) / W), 128 + 100 * Math.sin((2 * Math.PI * i) / W), 4 * j]);
const STEEP = (100 * 2 * Math.PI) / W;
const noise = picture((i, j) => [((i * 73 + j * 151) * 2654435761) % 251, ((i * 31 + j * 17) * 40503) % 241, (i ^ j) * 3]);
const jet = (lat: number) => 2e-5 * Math.cos(6 * lat);
const eddies = (lat: number, lon: number): [number, number] => [3e-6 * Math.sin(4 * lon) * Math.cos(3 * lat), 2e-6 * Math.cos(5 * lon) * Math.cos(lat)];

type Case = { name: string; worst: number; pixels: number; exact?: boolean };
type Out = { backend?: string; cases?: Case[]; bench?: unknown; error?: string };
const w = window as unknown as { __done?: Out };

const compare = (name: string, gpu: Float32Array, cpu: Float32Array, exact = false): Case => {
  let worst = 0;
  let same = true;
  for (let i = 0; i < cpu.length; i++) {
    worst = Math.max(worst, Math.abs(gpu[i]! - cpu[i]!));
    if (gpu[i] !== cpu[i]) same = false;
  }
  return { name, worst, pixels: worst / STEEP, ...(exact ? { exact: same } : {}) };
};

/** For the screens: a banded face, carried by a day, two days and three, then a long exposure, on the GPU. */
async function show(r: WebGPURenderer) {
  const w = 256;
  const h = 128;
  const face = new Float32Array(w * h * 4);
  for (let j = 0; j < h; j++)
    for (let i = 0; i < w; i++) {
      const lat = ((j + 0.5) / h) * Math.PI - Math.PI / 2;
      const lon = ((i + 0.5) / w) * 2 * Math.PI;
      const band = 0.5 + 0.5 * Math.cos(12 * lat);
      const spot = Math.exp(-(((lat - 0.35) / 0.08) ** 2) - (((lon - 2) / 0.25) ** 2));
      const fleck = 0.08 * Math.sin(37 * lon + 11 * lat) * Math.sin(23 * lat);
      face.set([200 * band + 40 + 180 * spot + 60 * fleck, 160 * band + 50 + 40 * spot + 50 * fleck, 110 * band + 60 + 10 * spot, 255], (j * w + i) * 4);
    }
  // Winds of Jupiter's order (jets about 150 m/s, 2e-6 rad/s; eddies a tenth of that).
  const table = windTable(64, 32, (lat) => jet(lat) / 10, (lat, lon) => eddies(lat, lon).map((v) => v / 10) as [number, number]);
  const rows: [string, Promise<Float32Array>][] = [
    ["held (the last proper picture)", Promise.resolve(face)],
    ["carried a day", drawKit(r, face, w, h, table, { kind: "carry", dt_s: DAY })],
    ["carried three days", drawKit(r, face, w, h, table, { kind: "carry", dt_s: 3 * DAY })],
    ["long exposure of a day", drawKit(r, face, w, h, table, { kind: "exposure", span_s: DAY })],
    ["long exposure of a week (at the limit)", drawKit(r, face, w, h, windTable(64, 32, (lat) => 1e-4 * (0.5 + Math.cos(lat))), { kind: "exposure", span_s: 7 * DAY })],
  ];
  document.body.style.cssText = "margin:0;background:#111;color:#ddd;font:14px sans-serif;padding:12px";
  for (const [label, p] of rows) {
    const px = await p;
    const c = document.createElement("canvas");
    c.width = w;
    c.height = h;
    const img = c.getContext("2d")!.createImageData(w, h);
    // Row 0 is the south pole: drawn north up.
    for (let j = 0; j < h; j++) for (let i = 0; i < w; i++) for (let k = 0; k < 4; k++) img.data[((h - 1 - j) * w + i) * 4 + k] = Math.max(0, Math.min(255, Math.round(px[(j * w + i) * 4 + k]!)));
    c.getContext("2d")!.putImageData(img, 0, 0);
    c.style.cssText = "width:512px;height:256px;image-rendering:pixelated;display:block";
    const d = document.createElement("div");
    d.style.cssText = "display:inline-block;margin:6px";
    d.append(c, Object.assign(document.createElement("div"), { textContent: `${label} (${(r.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? "WebGPU" : "WebGL2"})` }));
    document.body.append(d);
  }
}

(async () => {
  const out: Out = {};
  try {
    const r = new WebGPURenderer({ canvas: document.getElementById("c") as HTMLCanvasElement, forceWebGL: location.search.includes("gl") });
    await r.init();
    out.backend = (r.backend as { isWebGPUBackend?: boolean }).isWebGPUBackend ? "webgpu" : "webgl2";
    const jets = windTable(32, 16, jet);
    const stirred = windTable(32, 16, jet, eddies, 16 * 3600);
    const fast = windTable(32, 16, (lat) => 1e-4 * (0.5 + Math.cos(lat)));
    const cases: Case[] = [];
    for (const src of [bands, noise]) cases.push(compare(`carry 0 s (${src === bands ? "bands" : "noise"})`, await drawKit(r, src, W, H, stirred, { kind: "carry", dt_s: 0 }), src, true));
    cases.push(compare("carry, a pure jet for 10 days", await drawKit(r, bands, W, H, jets, { kind: "carry", dt_s: 10 * DAY }), carryCpu(bands, W, H, jets, 10 * DAY)));
    for (const dt of [5 * 3600, -7 * 3600, 3 * DAY]) cases.push(compare(`carry with eddies, ${dt / 3600} h`, await drawKit(r, bands, W, H, stirred, { kind: "carry", dt_s: dt }), carryCpu(bands, W, H, stirred, dt)));
    cases.push(compare("exposure, a day", await drawKit(r, bands, W, H, stirred, { kind: "exposure", span_s: DAY }), exposureCpu(bands, W, H, stirred, DAY)));
    const limit = (2 * Math.PI) / (1e-4 * 0.5) + 1;
    cases.push(compare("exposure at the limit (row mean)", await drawKit(r, noise, W, H, fast, { kind: "exposure", span_s: limit }), exposureCpu(noise, W, H, fast, limit)));
    // The mean itself, as the twin works it out (the row mean, held exactly by the processor test).
    cases.push(compare("exposure at the limit against the mean", await drawKit(r, noise, W, H, fast, { kind: "exposure", span_s: limit }), (() => {
      const m = rowMeans(noise, W, H);
      const o = new Float32Array(noise.length);
      for (let j = 0; j < H; j++) for (let i = 0; i < W; i++) o.set(m.subarray(j * 4, j * 4 + 4), (j * W + i) * 4);
      return o;
    })()));
    for (const k of [0, 0.3, 1]) cases.push(compare(`fade ${k}`, await drawKit(r, bands, W, H, jets, { kind: "fade", other: noise, weight: k }), fadeCpu(bands, noise, k), k === 0 || k === 1));
    out.cases = cases;
    if (location.search.includes("show")) await show(r);
    if (location.search.includes("bench")) out.bench = await benchKit(r);
  } catch (e) {
    out.error = String((e as Error).stack ?? e);
  }
  w.__done = out;
})();
