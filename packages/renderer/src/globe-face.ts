// A world's True face on the Body globe (VP3 part 3): the same face the system chart wears (true-air.ts
// `readTrueFace`), laid onto the globe's samples, so a giant shows its cloud decks there rather than a grey stand-in
// for ground it does not have. One lookup for both tiers: the canvas tier and gpu-globe.ts draw the recoloured samples.
// A world with ground keeps the tiles' own colours and relief (its clouds are the Body globe's weather layer).
import type { TrueTexture } from "./true-look.ts";

type V3 = [number, number, number];

/**
 * The face's colour along a direction (body frame, pole +z), sRGB 0..255, bilinear. The face is laid out as
 * true-look.ts `equirect` lays it: pixel (i, j) looks along (-cos 2πu · sin θ, -sin 2πu · sin θ, cos θ), with
 * u = (i + ½)/w and θ = π(1 - (j + ½)/h); longitude wraps, latitude is held at the poles.
 */
export function faceSample(t: Pick<TrueTexture, "width" | "height" | "data">, d: readonly number[]): V3 {
  const w = t.width;
  const h = t.height;
  const l = Math.hypot(d[0]!, d[1]!, d[2]!) || 1;
  const z = Math.max(-1, Math.min(1, d[2]! / l));
  const theta = Math.acos(z);
  let u = Math.atan2(-d[1]!, -d[0]!) / (2 * Math.PI);
  if (u < 0) u += 1;
  const x = u * w - 0.5;
  const y = Math.min(h - 1, Math.max(0, (1 - theta / Math.PI) * h - 0.5));
  const i0 = Math.floor(x);
  const j0 = Math.floor(y);
  const fx = x - i0;
  const fy = y - j0;
  const j1 = Math.min(h - 1, j0 + 1);
  const at = (i: number, j: number, c: number) => t.data[(j * w + (((i % w) + w) % w)) * 4 + c]!;
  const out: V3 = [0, 0, 0];
  for (let c = 0; c < 3; c++) {
    const a = at(i0, j0, c) * (1 - fx) + at(i0 + 1, j0, c) * fx;
    const b = at(i0, j1, c) * (1 - fx) + at(i0 + 1, j1, c) * fx;
    out[c] = a * (1 - fy) + b * fy;
  }
  return out;
}

const toLin = (c: number) => {
  const x = c / 255;
  return x <= 0.04045 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4);
};

/**
 * The globe's samples wearing a face: each sample's colour from the face along its direction, its normal straight
 * up (decks have no relief) and its albedo the colour's luminance (so the physical lighting gives back the face's own
 * colour). Shaped like weather.ts `withWeather`, so both tiers take it unchanged.
 */
export function withFace<P extends { faces: ({ s: number; rgb: Uint8ClampedArray; normal: Float32Array; albedo: Float32Array } | null)[] }>(p: P, face: Pick<TrueTexture, "width" | "height" | "data">, dirOf: (face: number, u: number, v: number) => V3): P {
  const faces = p.faces.map((f, n) => {
    if (!f) return null;
    const s = f.s;
    const rgb = new Uint8ClampedArray(f.rgb.length);
    const normal = new Float32Array(f.normal.length);
    const albedo = new Float32Array(f.albedo.length);
    const span = 2 / s;
    for (let j = 0; j < s; j++)
      for (let i = 0; i < s; i++) {
        const k = j * s + i;
        const d = dirOf(n, -1 + (i + 0.5) * span, -1 + (j + 0.5) * span);
        const c = faceSample(face, d);
        rgb[k * 3] = c[0];
        rgb[k * 3 + 1] = c[1];
        rgb[k * 3 + 2] = c[2];
        const l = Math.hypot(d[0], d[1], d[2]) || 1;
        normal[k * 3] = d[0] / l;
        normal[k * 3 + 1] = d[1] / l;
        normal[k * 3 + 2] = d[2] / l;
        albedo[k] = Math.max(0.01, 0.2126 * toLin(c[0]) + 0.7152 * toLin(c[1]) + 0.0722 * toLin(c[2]));
      }
    return { ...f, rgb, normal, albedo };
  });
  return { ...p, faces };
}

