// A world's air in True as a shell round its globe (VP-A3): the rim against space and the soft terminator, drawn by
// the look-up tables of air-luts.ts (Hillaire 2020). Per pixel the view ray is marched through the shell in a few
// steps; at each, the sun's light reaching that point is one read of the transmittance table and the light scattered
// many times one read of the multiple-scattering table, so no step marches towards the light. The output is
// premultiplied: what is behind is dimmed by the view ray's transmittance, the scattered light added. The camera is at
// the origin (gpu.ts), so the ray's direction is the pixel's own position, and the march starts at the pixel.
import { CustomBlending, DataTexture, DataUtils, FrontSide, HalfFloatType, LinearFilter, OneFactor, OneMinusSrcAlphaFactor, RGBAFormat, ClampToEdgeWrapping, Vector3 } from "three";
import { MeshBasicNodeMaterial } from "three/webgpu";
import { Fn, Loop, clamp, dot, exp, float, length, max, min, normalize, positionWorld, select, sqrt, texture, uniform, vec2, vec3, vec4 } from "three/tsl";
import { MS_N, TRANS_H, TRANS_W, type AirLuts, type AirShape } from "./air-luts.ts";

function lutTexture(data: Float32Array, w: number, h: number): DataTexture {
  const half = new Uint16Array(data.length);
  for (let i = 0; i < data.length; i++) half[i] = DataUtils.toHalfFloat(data[i]!);
  const t = new DataTexture(half, w, h, RGBAFormat, HalfFloatType);
  t.magFilter = LinearFilter;
  t.minFilter = LinearFilter;
  t.wrapS = ClampToEdgeWrapping;
  t.wrapT = ClampToEdgeWrapping;
  t.needsUpdate = true;
  return t;
}

export interface AirShell {
  material: MeshBasicNodeMaterial;
  /** The shell's radius over the globe's. */
  top: number;
  /** Each frame: the globe's camera-relative centre and drawn radius, the unit direction to the star, the sun's strength. */
  set(centre: readonly number[], radius: number, toStar: readonly number[], sun: number): void;
  dispose(): void;
}

/** A shell material for one world's air. `steps` view-ray steps (8 on phones, 16 by default). */
export function makeAirShell(shape: AirShape, luts: AirLuts, steps = 16): AirShell {
  const tTex = lutTexture(luts.transmittance, TRANS_W, TRANS_H);
  const mTex = lutTexture(luts.multiple, MS_N, MS_N);
  const uC = uniform(new Vector3());
  const uR = uniform(1);
  const uL = uniform(new Vector3(0, 0, 1));
  const uSun = uniform(3);
  const uBeta = uniform(new Vector3(...shape.beta));
  const uH = uniform(shape.scale_height);
  const top = luts.top;
  const material = new MeshBasicNodeMaterial({ side: FrontSide, transparent: true, depthWrite: false });
  material.blending = CustomBlending;
  material.blendSrc = OneFactor;
  material.blendDst = OneMinusSrcAlphaFactor;
  material.blendSrcAlpha = OneFactor;
  material.blendDstAlpha = OneMinusSrcAlphaFactor;
  const N = Math.max(4, Math.round(steps));
  // Table coordinates: u from the zenith cosine (-1..1), v from the height (0..top-1), half-texel inset.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const uvOf = (mu: any, h: any, w: number, hh: number) =>
    vec2(clamp(mu.add(1).mul(0.5), 0, 1).mul((w - 1) / w).add(0.5 / w), clamp(h.div(top - 1), 0, 1).mul((hh - 1) / hh).add(0.5 / hh));
  material.outputNode = Fn(() => {
    // In the globe's own units, starting where the view ray enters the shell (this fragment of its front face), so
    // the numbers stay near 1 however far the camera is.
    const d = normalize(positionWorld);
    const o = positionWorld.sub(uC).div(uR);
    const b = dot(o, d);
    const oo = dot(o, o);
    const discG = b.mul(b).sub(oo.sub(1));
    const tG = select(discG.greaterThan(0), b.negate().sub(sqrt(max(discG, 0))), float(1e9));
    const discT = b.mul(b).sub(oo.sub(top * top));
    const tIn = float(0);
    const tOut = b.negate().add(sqrt(max(discT, 0)));
    const tEnd = min(select(tG.greaterThan(0), tG, float(1e9)), tOut);
    const ds = max(tEnd.sub(tIn), 0).div(N);
    const tau = vec3(0, 0, 0).toVar();
    const sum = vec3(0, 0, 0).toVar();
    const nu = dot(d, uL);
    const rayleigh = float(3 / (16 * Math.PI)).mul(nu.mul(nu).add(1));
    Loop(N, ({ i }) => {
      const p = o.add(d.mul(tIn.add(float(i).add(0.5).mul(ds))));
      const r = length(p);
      const h = max(r.sub(1), 0);
      const rho = exp(h.div(uH).negate());
      const sigma = uBeta.mul(rho);
      const muS = dot(p, uL).div(r);
      const tSun = texture(tTex, uvOf(muS, h, TRANS_W, TRANS_H)).xyz;
      const psi = texture(mTex, uvOf(muS, h, MS_N, MS_N)).xyz;
      const tv = exp(tau.add(sigma.mul(ds).mul(0.5)).negate());
      sum.addAssign(tv.mul(sigma).mul(tSun.mul(rayleigh).add(psi)).mul(ds));
      tau.addAssign(sigma.mul(ds));
    });
    const tv = exp(tau.negate());
    return vec4(sum.mul(uSun), float(1).sub(tv.x.add(tv.y).add(tv.z).div(3)));
  })();
  return {
    material,
    top,
    set(centre, radius, toStar, sun) {
      uC.value.set(centre[0]!, centre[1]!, centre[2]!);
      uR.value = radius;
      uL.value.set(toStar[0]!, toStar[1]!, toStar[2]!).normalize();
      uSun.value = sun;
    },
    dispose() {
      material.dispose();
      tTex.dispose();
      mTex.dispose();
    },
  };
}
