// The detail ladder on the GPU (VP6): the noises of ladder-noise.ts built as one three.js TSL node, summed over the
// rungs of ladder.ts. One shader source for WebGPU and WebGL2. The caller hands a point in metres measured from its
// anchor (small numbers: rendering rule 3) and each frame the rungs from `rungs()`; the node returns the layer's
// drawn detail there, which the caller adds to its own facts (a warp of a band's latitude, a brightness, a height).
//
//   const lad = ladderNode("curl", local_m, { normal });
//   material.colorNode = bands(lat.add(lad.node.dot(north).div(radius)));
//   // every frame:
//   lad.set(rungs(resolved, discScale(R, px, w), tier, t_s, seed), { axis: spinAxis, stretch: resolved.stretch });
import { Vector3, Vector4 } from "three";
import { Fn, Loop, abs, cos, dot, float, floor, If, int, max, min, sin, sqrt, step, uniform, uniformArray, vec3 } from "three/tsl";
import { MAX_RUNGS, FLOW_REACH, type NoiseKind, type Rung } from "./ladder.ts";
import { cell, curl, ridged, simplex, type Ops, type V3 } from "./ladder-noise.ts";

// TSL nodes carry their maths as methods; three's types for them are loose, so the node type stays loose here too.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type N = any;

/** The noises' operations as TSL nodes. */
export const TSL: Ops<N> = {
  k: (v) => float(v),
  add: (a, b) => a.add(b),
  sub: (a, b) => a.sub(b),
  mul: (a, b) => a.mul(b),
  floor: (a) => floor(a),
  abs: (a) => abs(a),
  min: (a, b) => min(a, b),
  max: (a, b) => max(a, b),
  step: (e, x) => step(e, x),
  sqrt: (a) => sqrt(a),
  sin: (a) => sin(a),
  cos: (a) => cos(a),
};

export interface LadderNode {
  readonly kind: NoiseKind;
  /** A float for simplex, flow, cell (F1) and ridged; for curl a vec3 displacement in metres along the surface. */
  readonly node: N;
  /**
   * This frame's rungs (at most MAX_RUNGS, coarsest first), and the stretch: features `stretch` times longer across
   * `axis` than along it (a giant's jets run across its spin axis, so axis is the spin axis).
   */
  set(rs: readonly Rung[], o?: { stretch?: number; axis?: readonly [number, number, number] }): void;
  /** How many rungs the node is drawing now. */
  readonly count: number;
}

/**
 * Build a ladder node. `at` is a vec3 node in metres from the caller's anchor (the anchor itself goes into each
 * rung's origin on the processor). `normal` (unit, vec3) is needed by curl; `along` (unit tangent, vec3) by flow,
 * the direction the detail is carried.
 */
export function ladderNode(kind: NoiseKind, at: N, o: { normal?: N; along?: N } = {}): LadderNode {
  if (kind === "curl" && !o.normal) throw new Error("ladderNode: curl needs the surface normal");
  if (kind === "flow" && !o.along) throw new Error("ladderNode: flow needs the direction it is carried");
  // Per rung: a = (origin x, y, z, 1 / size); b = (weight × amplitude, alpha, phase, size).
  const uA = uniformArray(Array.from({ length: MAX_RUNGS }, () => new Vector4()), "vec4");
  const uB = uniformArray(Array.from({ length: MAX_RUNGS }, () => new Vector4()), "vec4");
  const uCount = uniform(0, "int");
  const uAxis = uniform(new Vector3(0, 0, 1));
  const uSquash = uniform(0);
  let count = 0;

  const node = Fn(() => {
    const sum = (kind === "curl" ? vec3(0, 0, 0) : float(0)).toVar();
    // The stretch, applied in the caller's small coordinates: the part along the axis is scaled by `stretch`
    // (features then run across it), p' = p + axis (axis · p)(stretch − 1).
    const p = at.add(uAxis.mul(dot(at, uAxis).mul(uSquash)));
    Loop({ start: int(0), end: int(MAX_RUNGS), type: "int", condition: "<" }, ({ i }: { i: N }) => {
      If(i.lessThan(uCount), () => {
        const a: N = uA.element(i);
        const b: N = uB.element(i);
        const q0 = p.mul(a.w).add(a.xyz);
        const q: V3<N> = [q0.x, q0.y, q0.z];
        if (kind === "simplex") sum.addAssign(simplex(TSL, q, b.y).value.mul(b.x));
        else if (kind === "ridged") sum.addAssign(ridged(TSL, q, b.y).mul(b.x));
        else if (kind === "cell") sum.addAssign(cell(TSL, q).f1.mul(b.x));
        else if (kind === "curl") {
          const n: N = o.normal;
          const c = curl(TSL, q, b.y, [n.x, n.y, n.z]);
          sum.addAssign(vec3(c[0], c[1], c[2]).mul(b.x.mul(b.w)));
        } else {
          // Flow: two samples carried along `along`, half a cycle apart, cross-faded (Vlachos 2010).
          const d: N = o.along;
          const ph0 = b.z;
          const ph1 = b.z.add(0.5).fract();
          const s0 = q0.sub(d.mul(ph0.sub(0.5).mul(FLOW_REACH * 2)));
          const s1 = q0.sub(d.mul(ph1.sub(0.5).mul(FLOW_REACH * 2)));
          const n0 = simplex(TSL, [s0.x, s0.y, s0.z], b.y).value;
          const n1 = simplex(TSL, [s1.x, s1.y, s1.z], b.y).value;
          const w0 = float(1).sub(abs(ph0.mul(2).sub(1)));
          sum.addAssign(n0.mul(w0).add(n1.mul(float(1).sub(w0))).mul(b.x));
        }
      });
    });
    return sum;
  })();

  return {
    kind,
    node,
    get count() {
      return count;
    },
    set(rs, opt = {}) {
      count = Math.min(MAX_RUNGS, rs.length);
      for (let i = 0; i < MAX_RUNGS; i++) {
        const r = rs[i];
        const A = uA.array[i] as Vector4;
        const B = uB.array[i] as Vector4;
        if (!r || i >= count) {
          A.set(0, 0, 0, 0);
          B.set(0, 0, 0, 0);
          continue;
        }
        A.set(r.origin[0], r.origin[1], r.origin[2], 1 / r.size_m);
        B.set(r.weight * r.amplitude, r.alpha, r.phase, r.size_m);
      }
      uCount.value = count;
      const ax = opt.axis ?? [0, 0, 1];
      const len = Math.hypot(ax[0], ax[1], ax[2]) || 1;
      uAxis.value.set(ax[0] / len, ax[1] / len, ax[2] / len);
      uSquash.value = (opt.stretch ?? 1) - 1;
    },
  };
}

