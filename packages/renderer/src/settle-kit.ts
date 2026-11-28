// The settle kit (FL2, SSE4-visual-flourish-design.md 4.4): three small shader pieces every stand-in is drawn with.
//
// - Carry: the held picture moved on along a wind table by the clock seconds since it was made: rows slide at their
//   jets' turn (exact for any time), the turning part runs as two overlapping loops, blended. Three texture reads a
//   pixel besides the table's.
// - Exposure: a long exposure along the same table over the time one picture stands for; up to eight reads, and the
//   row's mean (one column worked out once per picture) at the limit.
// - Fade: the settle and the seam, two pictures and one weight.
//
// One TSL source for WebGPU and WebGL2. Each piece is a node builder taking a texel place, so a view's own material
// can use it (FL3, FL5), and a material that draws it into a float target for the tests and for baking. Every sum is
// the one settle-kit-cpu.ts does, which tests/ui/settle-kit.e2e.mjs holds it to on both backends. Places are in the
// face's texels throughout, so a carry of zero seconds reads each texel at its own centre: the picture is untouched.
// Presentation only: what this draws is never read back into the kernel, a save or a postcard.

import { DataTexture, FloatType, Mesh, NearestFilter, NoBlending, OrthographicCamera, PlaneGeometry, RGBAFormat, RenderTarget, Scene, ClampToEdgeWrapping } from "three";
import { MeshBasicNodeMaterial, type WebGPURenderer } from "three/webgpu";
import { Fn, abs, clamp, float, floor, int, ivec2, min, screenCoordinate, select, sign, textureLoad, uniform, vec2, vec4 } from "three/tsl";
import type { WindTable } from "./wind-table.ts";
import { constant } from "../../ui-contract/src/index.ts";
import { EXPOSURE_TAPS, rowMeans } from "./settle-kit-cpu.ts";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type N = any;
const TAU = 2 * Math.PI;

/** A float RGBA texture read by index, never filtered or flipped. */
export function kitTexture(data: Float32Array, width: number, height: number): DataTexture {
  const tex = new DataTexture(data, width, height, RGBAFormat, FloatType);
  tex.magFilter = NearestFilter;
  tex.minFilter = NearestFilter;
  tex.wrapS = ClampToEdgeWrapping;
  tex.wrapT = ClampToEdgeWrapping;
  tex.generateMipmaps = false;
  tex.needsUpdate = true;
  return tex;
}

/** One texel by index. The texture is data, so three.js's flip check is left out (as stirred-gpu.ts does). */
const load = (tex: DataTexture, x: N, y: N): N => {
  const node = textureLoad(tex, ivec2(int(x), int(y)));
  (node as unknown as { setupUV: (b: unknown, uv: N) => N }).setupUV = (_b, uv) => uv;
  return node;
};

/** Two values and a share, written out (a · (1 − t) + b · t) so t = 0 gives a exactly. */
const lerp = (a: N, b: N, t: N): N => a.mul(float(1).sub(t)).add(b.mul(t));

/**
 * A `w` × `h` texture's colour at texel place (x, y), bilinear, wrapped east-west, held at the poles; the top row reads
 * itself (settle-kit-cpu.ts `sampleAt`).
 */
export function bilinear(tex: DataTexture, w: number, h: number, x: N, y: N): N {
  const W = float(w);
  const fy = clamp(y, 0, h - 1);
  const x0 = floor(x);
  const y0 = min(floor(fy), float(h - 1));
  const tx = x.sub(x0);
  const ty = fy.sub(y0);
  const xa = x0.sub(W.mul(floor(x0.div(W))));
  const xb1 = xa.add(1);
  const xb = select(xb1.greaterThanEqual(W), float(0), xb1);
  const yb = min(y0.add(1), float(h - 1));
  return lerp(lerp(load(tex, xa, y0), load(tex, xb, y0), tx), lerp(load(tex, xa, yb), load(tex, xb, yb), tx), ty);
}

/** The wind table's values at a face's texel place (wind-table.ts `windAt`). */
export function windNode(table: DataTexture, t: Pick<WindTable, "width" | "height">, w: number, h: number, x: N, y: N): N {
  const tx = x.add(0.5).div(w).mul(t.width).sub(0.5);
  const ty = clamp(y.add(0.5).div(h).mul(t.height).sub(0.5), 0, t.height - 1);
  return bilinear(table, t.width, t.height, tx, ty);
}

/** What a piece reads: the held picture, its size, the table and its size. */
export interface KitInputs {
  held: DataTexture;
  width: number;
  height: number;
  table: DataTexture;
  wind: Pick<WindTable, "width" | "height" | "loop_s">;
}

/** A loop's offset from its middle and its weight at `dt` (settle-kit-cpu.ts `loopPhases`). */
const phase = (dt: N, L: N, k: 0 | 1): [N, N] => {
  const t = dt.add(L.mul(k / 2));
  const tau = t.sub(L.mul(floor(t.div(L))));
  return [tau.sub(L.mul(0.5)), float(1).sub(abs(tau.mul(2).div(L).sub(1)))];
};

/** Carry as a node: the held picture at texel (x, y), carried on by `dt` clock seconds (a float node). */
export function carryNode(k: KitInputs, x: N, y: N, dt: N): N {
  const px = k.width / TAU;
  const py = k.height / Math.PI;
  const L = float(k.wind.loop_s);
  const here = windNode(k.table, k.wind, k.width, k.height, x, y);
  const x0 = x.sub(here.x.mul(dt).mul(px));
  const turn = windNode(k.table, k.wind, k.width, k.height, x0, y);
  const [s0, w0] = phase(dt, L, 0);
  const [s1, w1] = phase(dt, L, 1);
  const a = bilinear(k.held, k.width, k.height, x0.sub(turn.y.mul(s0).mul(px)), y.sub(turn.z.mul(s0).mul(py)));
  const b = bilinear(k.held, k.width, k.height, x0.sub(turn.y.mul(s1).mul(px)), y.sub(turn.z.mul(s1).mul(py)));
  // A loop of no weight is left out, not multiplied by nought, as the processor twin does.
  return select(w0.greaterThan(0), a.mul(w0), vec4(0)).add(select(w1.greaterThan(0), b.mul(w1), vec4(0)));
}

/** Exposure as a node: a long exposure of `span` clock seconds at texel (x, y); `means` is the rows' means (h × 1). */
export function exposureNode(k: KitInputs, means: DataTexture, x: N, y: N, span: N): N {
  const px = k.width / TAU;
  const py = k.height / Math.PI;
  const here = windNode(k.table, k.wind, k.width, k.height, x, y);
  const m = clamp(abs(here.x.add(here.y)).mul(abs(span)).div(TAU).mul(2).sub(1), 0, 1);
  const half = float(k.wind.loop_s / 2);
  let acc: N = vec4(0);
  for (let i = 0; i < EXPOSURE_TAPS; i++) {
    const tk = span.mul((i + 0.5) / EXPOSURE_TAPS);
    // The jets for the whole stretch, the storms and eddies for at most half a loop (settle-kit-cpu.ts).
    const tt = sign(tk).mul(min(abs(tk), half));
    acc = acc.add(bilinear(k.held, k.width, k.height, x.sub(here.x.mul(tk).add(here.y.mul(tt)).mul(px)), y.sub(here.z.mul(tt).mul(py))));
  }
  const taps = acc.mul(float(1).sub(m).div(EXPOSURE_TAPS));
  const mean = load(means, int(0), clamp(floor(y), 0, k.height - 1));
  return select(m.lessThan(1), taps, vec4(0)).add(mean.mul(m));
}

/** Fade as a node: `b` over `a` by `weight` (0 to 1). */
export function fadeNode(a: N, b: N, weight: N): N {
  const w = clamp(weight, 0, 1);
  return lerp(a, b, w);
}

/** A material whose output is the numbers given, untouched by colour, blending or alpha. */
function raw(): MeshBasicNodeMaterial {
  const m = new MeshBasicNodeMaterial();
  m.blending = NoBlending;
  m.depthTest = false;
  m.depthWrite = false;
  m.toneMapped = false;
  return m;
}

/** What to draw with the kit into a float picture of the held picture's size. */
export type KitDraw =
  | { kind: "carry"; dt_s: number }
  | { kind: "exposure"; span_s: number }
  | { kind: "fade"; other: Float32Array; weight: number };

/**
 * Draw one piece over a whole face on the GPU and read it back (RGBA floats, row 0 at the south pole): for the tests,
 * for the bench, and for a use that bakes its stand-in once (a still carried on between rebuilds).
 */
export async function drawKit(renderer: WebGPURenderer, held: Float32Array, width: number, height: number, table: WindTable, what: KitDraw): Promise<Float32Array> {
  const heldTex = kitTexture(held, width, height);
  const tableTex = kitTexture(table.data, table.width, table.height);
  const extra: DataTexture[] = [];
  const k: KitInputs = { held: heldTex, width, height, table: tableTex, wind: table };
  const m = raw();
  const ij = () => vec2(floor(screenCoordinate.x), floor(screenCoordinate.y));
  if (what.kind === "carry") {
    const dt = uniform(what.dt_s);
    m.fragmentNode = Fn(() => {
      const p = ij();
      return carryNode(k, p.x, p.y, dt);
    })();
  } else if (what.kind === "exposure") {
    const means = kitTexture(rowMeans(held, width, height), 1, height);
    extra.push(means);
    const span = uniform(what.span_s);
    m.fragmentNode = Fn(() => {
      const p = ij();
      return exposureNode(k, means, p.x, p.y, span);
    })();
  } else {
    const other = kitTexture(what.other, width, height);
    extra.push(other);
    const wgt = uniform(what.weight);
    m.fragmentNode = Fn(() => {
      const p = ij();
      return fadeNode(load(heldTex, p.x, p.y), load(other, p.x, p.y), wgt);
    })();
  }
  const target = new RenderTarget(width, height, { type: FloatType, format: RGBAFormat, minFilter: NearestFilter, magFilter: NearestFilter, depthBuffer: false });
  const scene = new Scene();
  const quad = new Mesh(new PlaneGeometry(2, 2), m);
  scene.add(quad);
  const cam = new OrthographicCamera(-1, 1, 1, -1, 0, 1);
  cam.position.z = 0.5;
  const was = renderer.getRenderTarget();
  try {
    renderer.setRenderTarget(target);
    renderer.render(scene, cam);
    const px = (await renderer.readRenderTargetPixelsAsync(target, 0, 0, width, height)) as Float32Array;
    // A target reads back from the bottom row up on WebGL, while the shader counts rows from the top as WebGPU does.
    if (!(renderer.backend as { isWebGLBackend?: boolean }).isWebGLBackend) return Float32Array.from(px);
    const out = new Float32Array(px.length);
    const row = width * 4;
    for (let j = 0; j < height; j++) out.set(px.subarray((height - 1 - j) * row, (height - j) * row), j * row);
    return out;
  } finally {
    renderer.setRenderTarget(was);
    m.dispose();
    quad.geometry.dispose();
    target.dispose();
    heldTex.dispose();
    tableTex.dispose();
    for (const x of extra) x.dispose();
  }
}

/** How long each piece takes on this device at a face size, ms (the bench's rows): median of `runs`, read back. */
export async function benchKit(renderer: WebGPURenderer, width = 512, height = 256, runs = 9): Promise<{ piece: KitDraw["kind"]; width: number; height: number; ms: number }[]> {
  const held = new Float32Array(width * height * 4);
  for (let i = 0; i < held.length; i++) held[i] = (i * 2654435761) % 251;
  const { windTable } = await import("./wind-table.ts");
  const table = windTable(256, 128, (lat) => 1e-5 * Math.cos(3 * lat), (lat, lon) => [2e-6 * Math.sin(5 * lon) * Math.cos(lat), 1e-6 * Math.cos(4 * lon)]);
  const out: { piece: KitDraw["kind"]; width: number; height: number; ms: number }[] = [];
  const day = constant("day");
  const draws: KitDraw[] = [{ kind: "carry", dt_s: day }, { kind: "exposure", span_s: day }, { kind: "fade", other: held, weight: 0.5 }];
  for (const d of draws) {
    await drawKit(renderer, held, width, height, table, d);
    const ms: number[] = [];
    for (let i = 0; i < runs; i++) {
      const t0 = performance.now();
      await drawKit(renderer, held, width, height, table, d);
      ms.push(performance.now() - t0);
    }
    ms.sort((a, b) => a - b);
    out.push({ piece: d.kind, width, height, ms: Math.round(ms[Math.floor(runs / 2)]! * 100) / 100 });
  }
  return out;
}
