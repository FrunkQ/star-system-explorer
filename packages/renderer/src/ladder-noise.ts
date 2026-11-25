// The detail ladder's noises (VP6), written once over a small set of maths operations so the same code runs on the
// processor (plain numbers: tests, the canvas tier, CPU bakes) and builds the GPU shader (three.js TSL nodes,
// ladder-node.ts). Nothing here imports three.js.
//
// Four kinds, all written fresh for SSE4 from the published methods (no code copied):
// - simplex: 3D simplex noise (Perlin 2001, as set out by Gustavson 2005; the branch-free corner ranking and the permutation-polynomial hash of
//   McEwan, Sheets, Gustavson and Richardson 2012), with each corner's gradient turned by an angle (flow noise,
//   Perlin and Neyret 2001), and its analytic gradient.
// - curl: the surface curl of that noise, n × ∇ψ, a swirl with no sources or sinks along the surface (Bridson,
//   Hourihan and Nordenstam 2007).
// - cell: distance to the nearest of one jittered point per lattice cell (Worley 1996), F1 and F2.
// - ridged: (1 − |simplex|)², sharp crests.
// The hash repeats every 289 lattice cells. Simplex works on a skewed lattice, so its own repeat is not along the
// axes; both simplex and cells repeat every 867 cells (3 × 289) along each axis, so a domain point may be taken
// modulo 867 on the processor in f64 before it reaches the GPU (ladder.ts `rungOrigin`): no large number is ever
// drawn with.

export const NOISE_REFERENCES = [
  { text: "Gustavson, S. (2005). Simplex noise demystified. Linköping University (the simplex lattice in 3D: skew by (√4 − 1)/3, unskew by 1/6, four corners, radial kernels).", url: "https://weber.itn.liu.se/~stegu/simplexnoise/simplexnoise.pdf" },
  { text: "McEwan, I., Sheets, D., Richardson, M. and Gustavson, S. (2012). Efficient computational noise in GLSL. Journal of Graphics Tools 16(2), 85-94 (branch-free simplex corner ranking, permutation polynomial (34x² + x) mod 289 as a hash).", doi: "10.1080/2151237X.2012.649621" },
  { text: "Perlin, K. and Neyret, F. (2001). Flow noise. SIGGRAPH 2001 Technical Sketches (gradients turned over time give swirling, unsmeared detail).", url: "https://www-evasion.imag.fr/Publications/2001/PN01/" },
  { text: "Bridson, R., Hourihan, J. and Nordenstam, M. (2007). Curl-noise for procedural fluid flow. ACM Transactions on Graphics 26(3), 46 (a velocity field as the curl of a noise potential has no divergence).", doi: "10.1145/1276377.1276435" },
  { text: "Worley, S. (1996). A cellular texture basis function. SIGGRAPH 96, 291-294 (distance to the nearest feature points F1 and F2).", doi: "10.1145/237170.237267" },
  { text: "Vlachos, A. (2010). Water flow in Portal 2. SIGGRAPH 2010 Advances in Real-Time Rendering (two-phase flow maps: two samples half a cycle apart, cross-faded, so advected detail never smears).", url: "https://advances.realtimerendering.com/s2010/index.html" },
];

/** The operations the noises are written in. `S` is a number on the processor, a TSL float node on the GPU. */
export interface Ops<S> {
  k(v: number): S;
  add(a: S, b: S): S;
  sub(a: S, b: S): S;
  mul(a: S, b: S): S;
  floor(a: S): S;
  abs(a: S): S;
  min(a: S, b: S): S;
  max(a: S, b: S): S;
  /** 0 when x < edge, else 1 (GLSL step). */
  step(edge: S, x: S): S;
  sqrt(a: S): S;
  sin(a: S): S;
  cos(a: S): S;
}

export type V3<S> = [S, S, S];

/** Plain numbers. */
export const NUM: Ops<number> = {
  k: (v) => v,
  add: (a, b) => a + b,
  sub: (a, b) => a - b,
  mul: (a, b) => a * b,
  floor: Math.floor,
  abs: Math.abs,
  min: Math.min,
  max: Math.max,
  step: (e, x) => (x < e ? 0 : 1),
  sqrt: Math.sqrt,
  sin: Math.sin,
  cos: Math.cos,
};

/** The hash's period in lattice cells. */
export const PERIOD = 289;
/**
 * The period every noise here shares along each axis, in lattice cells. Simplex's lattice is skewed by s = (x+y+z)/3,
 * so a shift by 289 along one axis is not a repeat; a shift v of whole 289s is one when v's components sum to a
 * multiple of 867, which (867, 0, 0) does.
 */
export const WRAP = 3 * PERIOD;
/** Scales simplex sums to within ±1 (measured over 300,000 points: |n| reached 0.92 at most, standard deviation 0.36, mean 0.006). */
export const SIMPLEX_SCALE = 100;
const GOLDEN = Math.PI * (3 - Math.sqrt(5));

function mod289<S>(o: Ops<S>, x: S): S {
  return o.sub(x, o.mul(o.floor(o.mul(x, o.k(1 / PERIOD))), o.k(PERIOD)));
}
/** (34x² + 10x) mod 289: exact in float32 for 0 ≤ x < 289 (the largest product is under 2.9 million). */
function permute<S>(o: Ops<S>, x: S): S {
  return mod289(o, o.mul(o.add(o.mul(x, o.k(34)), o.k(10)), x));
}
/** A lattice point's hash, 0..288. */
function hash3<S>(o: Ops<S>, i: S, j: S, k: S): S {
  return permute(o, o.add(permute(o, o.add(permute(o, mod289(o, k)), mod289(o, j))), mod289(o, i)));
}

/**
 * The gradient at a lattice point of hash h, turned by `alpha`: a point of a Fibonacci spiral on the unit sphere
 * (an even spread of directions) swung by alpha towards the spiral's eastward tangent. Unit length.
 */
function gradient<S>(o: Ops<S>, h: S, ca: S, sa: S): V3<S> {
  const z = o.sub(o.k(1), o.mul(o.add(h, o.k(0.5)), o.k(2 / PERIOD)));
  const r = o.sqrt(o.max(o.sub(o.k(1), o.mul(z, z)), o.k(0)));
  const phi = o.mul(h, o.k(GOLDEN));
  const cp = o.cos(phi);
  const sp = o.sin(phi);
  // g1 = (r cos φ, r sin φ, z); t1 = (−sin φ, cos φ, 0); g = cos α g1 + sin α t1.
  return [o.sub(o.mul(ca, o.mul(r, cp)), o.mul(sa, sp)), o.add(o.mul(ca, o.mul(r, sp)), o.mul(sa, cp)), o.mul(ca, z)];
}

const dot3 = <S>(o: Ops<S>, a: V3<S>, b: V3<S>): S => o.add(o.add(o.mul(a[0], b[0]), o.mul(a[1], b[1])), o.mul(a[2], b[2]));

/**
 * Simplex noise at q (lattice units) with its gradients turned by alpha (radians; 0 is plain simplex). Returns the
 * value (about ±1) and its gradient d value / d q. The kernel of each corner reaches 0.5 in squared distance, so the
 * noise is continuous with continuous first derivatives everywhere.
 */
export function simplex<S>(o: Ops<S>, q: V3<S>, alpha: S): { value: S; grad: V3<S> } {
  const F3 = o.k(1 / 3);
  const G3 = 1 / 6;
  const s = o.mul(o.add(o.add(q[0], q[1]), q[2]), F3);
  const i = o.floor(o.add(q[0], s));
  const j = o.floor(o.add(q[1], s));
  const k = o.floor(o.add(q[2], s));
  const t = o.mul(o.add(o.add(i, j), k), o.k(G3));
  const x0: V3<S> = [o.sub(q[0], o.sub(i, t)), o.sub(q[1], o.sub(j, t)), o.sub(q[2], o.sub(k, t))];
  // Branch-free ranking of the corner offsets (which simplex of the six in the cube).
  const gx = o.step(x0[1], x0[0]); // x ≥ y
  const gy = o.step(x0[2], x0[1]); // y ≥ z
  const gz = o.step(x0[0], x0[2]); // z ≥ x
  const lx = o.sub(o.k(1), gx);
  const ly = o.sub(o.k(1), gy);
  const lz = o.sub(o.k(1), gz);
  const i1: V3<S> = [o.min(gx, lz), o.min(gy, lx), o.min(gz, ly)];
  const i2: V3<S> = [o.max(gx, lz), o.max(gy, lx), o.max(gz, ly)];
  const corner = (off: V3<S>, c: number): V3<S> => [o.add(o.sub(x0[0], off[0]), o.k(c * G3)), o.add(o.sub(x0[1], off[1]), o.k(c * G3)), o.add(o.sub(x0[2], off[2]), o.k(c * G3))];
  const one = o.k(1);
  const zero = o.k(0);
  const offs: V3<S>[] = [[zero, zero, zero], i1, i2, [one, one, one]];
  const ca = o.cos(alpha);
  const sa = o.sin(alpha);
  let value = o.k(0);
  let g0 = o.k(0);
  let g1 = o.k(0);
  let g2 = o.k(0);
  for (let c = 0; c < 4; c++) {
    const off = offs[c]!;
    const x = c === 0 ? x0 : corner(off, c);
    const h = hash3(o, o.add(i, off[0]), o.add(j, off[1]), o.add(k, off[2]));
    const g = gradient(o, h, ca, sa);
    const w = o.max(o.sub(o.k(0.5), dot3(o, x, x)), o.k(0));
    const w2 = o.mul(w, w);
    const w3 = o.mul(w2, w);
    const w4 = o.mul(w2, w2);
    const gd = dot3(o, g, x);
    value = o.add(value, o.mul(w4, gd));
    // d/dx [w⁴ (g·x)] = w⁴ g − 8 w³ (g·x) x
    const m = o.mul(o.k(8), o.mul(w3, gd));
    g0 = o.add(g0, o.sub(o.mul(w4, g[0]), o.mul(m, x[0])));
    g1 = o.add(g1, o.sub(o.mul(w4, g[1]), o.mul(m, x[1])));
    g2 = o.add(g2, o.sub(o.mul(w4, g[2]), o.mul(m, x[2])));
  }
  const K = o.k(SIMPLEX_SCALE);
  return { value: o.mul(value, K), grad: [o.mul(g0, K), o.mul(g1, K), o.mul(g2, K)] };
}

/** The surface curl of the noise: n × ∇ψ, for a unit surface normal n. Tangent to the surface, divergence-free along it. */
export function curl<S>(o: Ops<S>, q: V3<S>, alpha: S, n: V3<S>): V3<S> {
  const { grad: g } = simplex(o, q, alpha);
  return [o.sub(o.mul(n[1], g[2]), o.mul(n[2], g[1])), o.sub(o.mul(n[2], g[0]), o.mul(n[0], g[2])), o.sub(o.mul(n[0], g[1]), o.mul(n[1], g[0]))];
}

/** Sharp crests: (1 − |simplex|)², 0..1. */
export function ridged<S>(o: Ops<S>, q: V3<S>, alpha: S): S {
  const r = o.sub(o.k(1), o.abs(simplex(o, q, alpha).value));
  return o.mul(r, r);
}

/**
 * Cells (Worley F1 and F2): distances from q to the nearest and second-nearest feature point, one point per lattice
 * cell jittered across 0.9 of it, searched over the 27 cells round q's own. In lattice units: F1 is 0 to about 1.1, mean about 0.52.
 */
export function cell<S>(o: Ops<S>, q: V3<S>): { f1: S; f2: S } {
  const i = o.floor(q[0]);
  const j = o.floor(q[1]);
  const k = o.floor(q[2]);
  const fx = o.sub(q[0], i);
  const fy = o.sub(q[1], j);
  const fz = o.sub(q[2], k);
  let f1 = o.k(9);
  let f2 = o.k(9);
  const J = 0.9 / PERIOD;
  for (let a = -1; a <= 1; a++) {
    for (let b = -1; b <= 1; b++) {
      for (let c = -1; c <= 1; c++) {
        const h = hash3(o, o.add(i, o.k(a)), o.add(j, o.k(b)), o.add(k, o.k(c)));
        const hy = permute(o, h);
        const hz = permute(o, hy);
        const dx = o.sub(o.add(o.k(a + 0.05), o.mul(h, o.k(J))), fx);
        const dy = o.sub(o.add(o.k(b + 0.05), o.mul(hy, o.k(J))), fy);
        const dz = o.sub(o.add(o.k(c + 0.05), o.mul(hz, o.k(J))), fz);
        const d = o.add(o.add(o.mul(dx, dx), o.mul(dy, dy)), o.mul(dz, dz));
        // Keep the two smallest without branches.
        f2 = o.min(f2, o.max(f1, d));
        f1 = o.min(f1, d);
      }
    }
  }
  return { f1: o.sqrt(f1), f2: o.sqrt(f2) };
}
