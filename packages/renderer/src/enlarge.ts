// Enlarging on Light (VP5, SSE4-visual-polish-design.md 5.1 and 8.4): a view drawn at a fraction of its pixels is
// brought up to full size by a sharp bicubic (Catmull-Rom, nine bilinear taps) and then sharpened by a robust
// contrast-adaptive sharpener in the manner of AMD FidelityFX FSR1's RCAS (MIT licence; the method written fresh here
// from its published description, not ported). Two TSL functions the shared finish (finish.ts) chains in its output:
// `enlargeNode` reads the small picture at a full-size pixel, `sharpenNode` sharpens the full-size result. A camera
// effect, not the world: it adds no detail, it only keeps the edges that were drawn.
//
// `rcasLobe` is the sharpener's weight in plain numbers, the same formula as the shader, so the maths is tested
// without a GPU.
import { clamp, exp2, float, max, min, vec2, vec4 } from "three/tsl";

/** The sharpener's strongest lobe: a quarter less a sixteenth, as FSR1's RCAS limit, so it never rings. */
export const RCAS_LIMIT = 0.25 - 1 / 16;

/**
 * The sharpening weight for a pixel from its four neighbours' lowest and highest values (0 to 1) and the sharpness in
 * stops (0 is the strongest; each stop halves it). Negative: the neighbours are taken away, which sharpens.
 */
export function rcasLobe(min4: number, max4: number, stops: number): number {
  const hitMin = min4 / (4 * Math.max(max4, 1e-6));
  const hitMax = (1 - max4) / Math.min(4 * min4 - 4, -1e-6);
  const lobe = Math.max(-hitMin, hitMax);
  return Math.max(-RCAS_LIMIT, Math.min(lobe, 0)) * Math.pow(2, -stops);
}

/** One sharpened value from the centre and its four neighbours, kept within the five's range so a lone peak never rings. */
export function rcasValue(e: number, b: number, d: number, f: number, h: number, stops: number): number {
  const mn = Math.min(b, d, f, h);
  const mx = Math.max(b, d, f, h);
  const lobe = rcasLobe(mn, mx, stops);
  const v = (lobe * (b + d + f + h) + e) / (4 * lobe + 1);
  return Math.min(Math.max(v, Math.min(mn, e)), Math.max(mx, e));
}

/** Catmull-Rom weights for a sample `t` (0 to 1) past the texel centre: w0 to w3. */
export function catmullRom(t: number): [number, number, number, number] {
  const t2 = t * t;
  const t3 = t2 * t;
  return [-0.5 * t3 + t2 - 0.5 * t, 1.5 * t3 - 2.5 * t2 + 1, -1.5 * t3 + 2 * t2 + 0.5 * t, 0.5 * t3 - 0.5 * t2];
}

// TSL nodes are typed loosely here: the shapes vary by swizzle, and three's own types do not follow them through maths.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type N = any;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type Sampler = (uv: any) => any;

/**
 * Catmull-Rom enlarging in nine bilinear taps: `sample(uv)` reads the small picture with linear filtering, `size` is
 * its size in pixels, `uv` the full-size pixel's place (0 to 1).
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function enlargeNode(sample: Sampler, size: any, uv: any): N {
  const pos = uv.mul(size);
  const c = pos.sub(0.5).floor().add(0.5);
  const t = pos.sub(c);
  const t2 = t.mul(t);
  const t3 = t2.mul(t);
  const w0 = t3.mul(-0.5).add(t2).sub(t.mul(0.5));
  const w1 = t3.mul(1.5).sub(t2.mul(2.5)).add(1);
  const w2 = t3.mul(-1.5).add(t2.mul(2)).add(t.mul(0.5));
  const w3 = t3.mul(0.5).sub(t2.mul(0.5));
  const w12 = w1.add(w2);
  const o12 = w2.div(w12);
  const inv = vec2(1, 1).div(size);
  const p0 = c.sub(1).mul(inv);
  const p3 = c.add(2).mul(inv);
  const p12 = c.add(o12).mul(inv);
  const at = (x: N, y: N) => sample(vec2(x, y));
  const r = at(p0.x, p0.y).mul(w0.x.mul(w0.y))
    .add(at(p12.x, p0.y).mul(w12.x.mul(w0.y)))
    .add(at(p3.x, p0.y).mul(w3.x.mul(w0.y)))
    .add(at(p0.x, p12.y).mul(w0.x.mul(w12.y)))
    .add(at(p12.x, p12.y).mul(w12.x.mul(w12.y)))
    .add(at(p3.x, p12.y).mul(w3.x.mul(w12.y)))
    .add(at(p0.x, p3.y).mul(w0.x.mul(w3.y)))
    .add(at(p12.x, p3.y).mul(w12.x.mul(w3.y)))
    .add(at(p3.x, p3.y).mul(w3.x.mul(w3.y)));
  // Catmull-Rom overshoots a little at hard edges: keep it within the picture's range.
  return max(r, vec4(0, 0, 0, 0)) as N;
}

/**
 * The sharpener over the full-size picture: `sample(uv)` reads it, `texel` is one pixel in uv, `stops` the sharpness
 * (0 strongest, 0.2 FSR1's usual). Per channel, with one lobe from the green channel so colours do not drift.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function sharpenNode(sample: Sampler, texel: any, uv: any, stops: any): N {
  const e = sample(uv);
  const b = sample(uv.sub(vec2(0, texel.y)));
  const d = sample(uv.sub(vec2(texel.x, 0)));
  const f = sample(uv.add(vec2(texel.x, 0)));
  const h = sample(uv.add(vec2(0, texel.y)));
  const g = (x: N) => clamp(x.g, 0, 1);
  const mn = min(min(g(b), g(d)), min(g(f), g(h)));
  const mx = max(max(g(b), g(d)), max(g(f), g(h)));
  const hitMin = mn.div(max(mx.mul(4), 1e-6));
  const hitMax = float(1).sub(mx).div(min(mn.mul(4).sub(4), -1e-6));
  const lobe = clamp(max(hitMin.negate(), hitMax), -RCAS_LIMIT, 0).mul(exp2(float(stops).negate()));
  const v = b.add(d).add(f).add(h).mul(lobe).add(e).div(lobe.mul(4).add(1));
  // Kept within the five pixels' range, so a lone peak never rings.
  const lo = min(min(min(b, d), min(f, h)), e);
  const hi = max(max(max(b, d), max(f, h)), e);
  return clamp(v, lo, hi) as N;
}
