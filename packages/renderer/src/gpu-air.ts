// The atmosphere pass on the GPU (R2): single scattering by molecules
// (Rayleigh), ray-marched through a shell around the unit globe, after
// Nishita et al. (1993). Per pixel it marches the view ray from the shell to
// the ground (or out of the shell), and from each step towards the light,
// sums the light scattered towards the eye, and returns that light plus the
// view ray's transmittance, which dims the ground behind it. The planet's
// own shadow is honoured. Coefficients come from `atmosphere.ts` (physics
// from the kernel's facts), scaled to render units (globe radius 1).
// Multiple scattering (Hillaire 2020) is not done yet; it brightens thick
// skies and is listed in docs/fudges.md.
import { CustomBlending, OneFactor, OneMinusSrcAlphaFactor, FrontSide, Vector3 } from "three";
import { MeshBasicNodeMaterial } from "three/webgpu";
import { Fn, Loop, dot, exp, float, length, max, min, positionWorld, select, sqrt, uniform, vec3, vec4 } from "three/tsl";

export interface AirRender {
  /** Rayleigh scattering per render unit (globe radius 1) at the surface, red, green, blue. */
  beta: [number, number, number];
  /** Scale height in render units. */
  scale_height: number;
  /** Top of the shell in render units (1 + some scale heights). */
  top: number;
}

export function makeAirMaterial(steps: number, lightSteps: number) {
  const uBeta = uniform(new Vector3(0, 0, 0));
  const uH = uniform(0.0013);
  const uTop = uniform(1.01);
  const uL = uniform(new Vector3(0, 0, 1));
  const uSun = uniform(1);
  const material = new MeshBasicNodeMaterial({ side: FrontSide, transparent: true, depthWrite: false });
  material.blending = CustomBlending;
  material.blendSrc = OneFactor;
  // Premultiplied: what is behind is dimmed by the view ray's transmittance, the scattered light is added.
  material.blendDst = OneMinusSrcAlphaFactor;
  material.blendSrcAlpha = OneFactor;
  material.blendDstAlpha = OneMinusSrcAlphaFactor;
  const N = Math.max(4, Math.round(steps));
  const M = Math.max(2, Math.round(lightSteps));
  material.outputNode = Fn(() => {
    const o = positionWorld;
    const d = vec3(0, 0, -1); // the orthographic camera looks down -z
    const b = dot(o, d);
    const c1 = dot(o, o).sub(1);
    const disc1 = b.mul(b).sub(c1);
    const tGround = select(disc1.greaterThan(0), b.negate().sub(sqrt(max(disc1, 0))), float(1e9));
    const top2 = uTop.mul(uTop);
    const discT = b.mul(b).sub(dot(o, o).sub(top2));
    const tExit = b.negate().add(sqrt(max(discT, 0)));
    const tEnd = min(select(tGround.greaterThan(0), tGround, float(1e9)), tExit);
    const ds = max(tEnd, 0).div(N);
    const odView = float(0).toVar();
    const sum = vec3(0, 0, 0).toVar();
    Loop(N, ({ i }) => {
      const p = o.add(d.mul(float(i).add(0.5).mul(ds)));
      const rho = exp(length(p).sub(1).div(uH).negate());
      odView.addAssign(rho.mul(ds));
      const bl = dot(p, uL);
      const tl = bl.negate().add(sqrt(max(bl.mul(bl).sub(dot(p, p).sub(top2)), 0)));
      // In the planet's shadow when the ray to the light passes inside the globe.
      const shadow = bl.lessThan(0).and(dot(p, p).sub(bl.mul(bl)).lessThan(1));
      const dl = tl.div(M);
      const odL = float(0).toVar();
      Loop(M, ({ i: j }) => {
        const q = p.add(uL.mul(float(j).add(0.5).mul(dl)));
        odL.addAssign(exp(length(q).sub(1).div(uH).negate()).mul(dl));
      });
      const T = exp(uBeta.mul(odView.add(odL)).negate());
      sum.addAssign(select(shadow, vec3(0, 0, 0), T.mul(rho.mul(ds))));
    });
    const mu = dot(d, uL);
    const phase = float(3 / (16 * Math.PI)).mul(mu.mul(mu).add(1));
    const inscatter = sum.mul(uBeta).mul(phase).mul(uSun);
    const tv = exp(uBeta.mul(odView).negate());
    return vec4(inscatter, float(1).sub(tv.x.add(tv.y).add(tv.z).div(3)));
  })();
  return {
    material,
    steps: N,
    lightSteps: M,
    set(a: AirRender, L: readonly number[], sun: number) {
      uBeta.value.set(a.beta[0], a.beta[1], a.beta[2]);
      uH.value = a.scale_height;
      uTop.value = a.top;
      uL.value.set(L[0]!, L[1]!, L[2]!);
      uSun.value = sun;
    },
  };
}
