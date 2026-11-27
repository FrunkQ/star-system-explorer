// Shadows, eclipses and the sea's glint on the GPU (renderer audit items 1 and 2): the sums of eclipse.ts, per pixel,
// in one TSL source for WebGPU and WebGL2. A world's lit materials take a lighting model whose direct light (the star's)
// is first passed through what stands in the way: the bodies covering part of the star's disc (penumbra, umbra, an
// annular eclipse), leaving the umbra's own bent light, and the world's rings along the slant path. The ambient trace
// is not shadowed (it stands for light from everywhere). Where the ground is liquid the sun's glint off the wind-ruffled
// surface is added (Cox and Munk 1954). A ring's material takes the world's shadow across it by the same disc cover.
// Everything a pixel needs is in uniforms set each frame from the item (setShade), so the materials compile once.
import { DataTexture, RGBAFormat, UnsignedByteType, Vector3, Vector4 } from "three";
import { PhysicalLightingModel, type NodeMaterial } from "three/webgpu";
import { abs, acos, asin, atan, clamp, cross, dot, exp, float, Fn, length, max, min, normalize, normalView, positionViewDirection, positionWorld, pow, select, sqrt, texture, uniform, vec2, vec3 } from "three/tsl";
import { GLINT_FALLBACK, MAX_CASTERS, type Shade } from "./eclipse.ts";
import type { Vec3 } from "./frames.ts";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type N = any;

/** The star's angular radius used where a view does not give one: the Sun's from Earth (fudge register, "Shadows in True"). */
export const STAR_FALLBACK = 0.00465;

/** The share of a unit disc covered by a disc of radius r at d (eclipse.ts `discCover`), as a node. */
const discCover = Fn(([r0, d0]: [N, N]) => {
  const r = max(r0, 1e-6);
  const d = max(d0, 1e-6);
  const inside = select(r.greaterThanEqual(1), float(1), r.mul(r));
  const a1 = acos(clamp(d.mul(d).add(1).sub(r.mul(r)).div(d.mul(2)), -1, 1));
  const a2 = acos(clamp(d.mul(d).add(r.mul(r)).sub(1).div(d.mul(r).mul(2)), -1, 1));
  const k = sqrt(max(float(0), d.negate().add(1).add(r).mul(d.add(1).sub(r)).mul(d.sub(1).add(r)).mul(d.add(1).add(r))));
  const lens = clamp(a1.add(r.mul(r).mul(a2)).sub(k.mul(0.5)).div(Math.PI), 0, 1);
  return select(d.greaterThanEqual(r.add(1)), float(0), select(d.lessThanEqual(abs(float(1).sub(r))), inside, lens));
});

/** The cover of the star (angular radius `as`, toward `s`) by a ball of radius `rc` at `c`, seen from `p`. */
const coverBy = Fn(([p, c, rc, s, as]: [N, N, N, N, N]) => {
  const dv = c.sub(p);
  const dist = length(dv);
  const ac = asin(min(float(1), rc.div(max(dist, 1e-6))));
  const theta = atan(length(cross(dv, s)), dot(dv, s));
  // Inside the ball (or a slot left empty, radius 0) casts nothing.
  return select(dist.greaterThan(rc).and(rc.greaterThan(0)), discCover(ac.div(as), theta.div(as)), float(0));
});

const blank = () => {
  const t = new DataTexture(new Uint8Array([0, 0, 0, 0]), 1, 1, RGBAFormat, UnsignedByteType);
  t.needsUpdate = true;
  return t;
};

/** One world's shadow uniforms, shared by its surface, cloud and ring materials. */
export interface ShadeState {
  /** Set from the item each frame: where it is drawn, how big, the way to its star and what stands in the way. */
  set(o: { rel: Vec3; radius: number; toStar: Vec3; shade?: Shade; rings?: { normal: Vec3; inner: number; outer: number; tex: DataTexture } | null; glint?: { slope_variance?: number; normal_reflectance?: number } | null }): void;
  /** Give a lit material this world's shadows (and, with `wet`, its glint where `wet` is above zero). Once per material. */
  wear(m: NodeMaterial, wet?: N): void;
  /** The ring material's light: the world's shadow across it, times `scale` (the ring's own lighting). */
  ringLight(): N;
  readonly uRingScale: { value: number };
}

export function makeShade(): ShadeState {
  const uC = uniform(new Vector3());
  const uR = uniform(1);
  const uS = uniform(new Vector3(0, 0, 1));
  const uAs = uniform(STAR_FALLBACK);
  const uCast = Array.from({ length: MAX_CASTERS }, () => uniform(new Vector4(0, 0, 0, 0)));
  const uUmbra = Array.from({ length: MAX_CASTERS }, () => uniform(new Vector3(0, 0, 0)));
  const uRingN = uniform(new Vector3(0, 0, 1));
  const uRingIn = uniform(1);
  const uRingOut = uniform(1);
  const uRingOn = uniform(0);
  // The rings' profile: the item's own texture, set before its materials first compile (gpu.ts sets the shade first):
  // the 1-pixel placeholder swapped for the profile after compiling was not rebound on WebGL2. A later profile (a new
  // ring key, another width) swapped in by value is read at once (checked in the browser, EC2).
  let ringTex: DataTexture = blank();
  const uSigma2 = uniform(GLINT_FALLBACK.slope_variance);
  const uF0 = uniform(GLINT_FALLBACK.normal_reflectance);
  const uGlint = uniform(0);
  const uRingScale = uniform(1);
  const ringNodes: N[] = [];

  // Where a pixel is on the world, in its radii from its centre (camera-relative numbers only: rendering rule 3).
  const local = () => positionWorld.sub(uC).div(uR);
  const s = () => normalize(uS);

  /** The starlight reaching point p past the casters, per channel. */
  const pastCasters = (p: N): N => {
    let light: N = vec3(1, 1, 1);
    for (let i = 0; i < MAX_CASTERS; i++) {
      const c = uCast[i]!;
      const f = coverBy(p, c.xyz, c.w, s(), uAs);
      light = light.mul(vec3(1, 1, 1).sub(vec3(f, f, f)).add(uUmbra[i]!.mul(f)));
    }
    return light;
  };

  /** The share the rings let through to p (eclipse.ts `ringPassAt`). */
  const ringPass = (p: N): N => {
    const tex = texture(ringTex);
    ringNodes.push(tex);
    const n = normalize(uRingN);
    const mu0 = dot(s(), n);
    const t = p.dot(n).negate().div(select(abs(mu0).greaterThan(1e-4), mu0, float(1e-4)));
    const q = p.add(s().mul(t));
    const rho = length(q);
    const u = rho.sub(uRingIn).div(max(uRingOut.sub(uRingIn), 1e-6));
    const cover = clamp((tex.sample(vec2(clamp(u, 0, 1), 0.5)) as N).w, 0, 1);
    const pass = pow(max(float(1).sub(cover), 0), float(1).div(max(abs(mu0), 1e-4)));
    const crosses = t.greaterThan(0).and(u.greaterThanEqual(0)).and(u.lessThanEqual(1)).and(uRingOn.greaterThan(0));
    return select(crosses, pass, float(1));
  };

  class Shadowed extends PhysicalLightingModel {
    constructor(private readonly wet: N | null) {
      super();
    }
    direct(args: N, builder: N) {
      const p = local();
      const light = pastCasters(p).mul(ringPass(p));
      const lightColor = args.lightColor.mul(light);
      super.direct({ ...args, lightColor }, builder);
      if (!this.wet) return;
      // The glint (eclipse.ts `glintBrdf`), in view space: the facets tipped to mirror the star into the eye.
      const nn = normalView;
      const l = args.lightDirection;
      const v = positionViewDirection;
      const mu0 = max(dot(nn, l), 0);
      const mu = max(dot(nn, v), 1e-3);
      const h = normalize(l.add(v));
      const cb = max(dot(nn, h), 1e-3);
      const cb2 = cb.mul(cb);
      const tan2 = float(1).sub(cb2).div(cb2);
      const pdf = exp(tan2.div(uSigma2).negate()).div(uSigma2.mul(Math.PI));
      const fres = uF0.add(float(1).sub(uF0).mul(pow(float(1).sub(max(dot(v, h), 0)), 5)));
      const brdf = fres.mul(pdf).div(max(mu0, 1e-3).mul(mu).mul(cb2).mul(cb2).mul(4));
      args.reflectedLight.directSpecular.addAssign(lightColor.mul(mu0).mul(brdf).mul(this.wet).mul(uGlint));
    }
  }

  return {
    uRingScale,
    set(o) {
      uC.value.set(o.rel[0], o.rel[1], o.rel[2]);
      uR.value = o.radius;
      uS.value.set(o.toStar[0], o.toStar[1], o.toStar[2]);
      uAs.value = o.shade && o.shade.star > 0 ? o.shade.star : STAR_FALLBACK;
      const cs = o.shade?.casters ?? [];
      for (let i = 0; i < MAX_CASTERS; i++) {
        const c = cs[i];
        uCast[i]!.value.set(c?.at[0] ?? 0, c?.at[1] ?? 0, c?.at[2] ?? 0, c?.radius ?? 0);
        const u = c?.umbra ?? [0, 0, 0];
        uUmbra[i]!.value.set(u[0], u[1], u[2]);
      }
      const r = o.rings;
      uRingOn.value = r ? 1 : 0;
      if (r) {
        uRingN.value.set(r.normal[0], r.normal[1], r.normal[2]);
        uRingIn.value = r.inner;
        uRingOut.value = r.outer;
        if (ringTex !== r.tex) {
          ringTex = r.tex;
          for (const t of ringNodes) t.value = r.tex;
        }
      }
      uGlint.value = o.glint ? 1 : 0;
      if (o.glint) {
        uSigma2.value = o.glint.slope_variance ?? GLINT_FALLBACK.slope_variance;
        uF0.value = o.glint.normal_reflectance ?? GLINT_FALLBACK.normal_reflectance;
      }
    },
    wear(m, wet) {
      (m as unknown as { setupLightingModel: () => unknown }).setupLightingModel = () => new Shadowed(wet ?? null);
      m.needsUpdate = true;
    },
    ringLight() {
      // A ring's particle: the world (radius 1 at the centre) covering the star as seen from it.
      const f = coverBy(local(), vec3(0, 0, 0), float(1), s(), uAs);
      return float(1).sub(f).mul(uRingScale);
    },
  };
}
