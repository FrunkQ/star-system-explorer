/**
 * sRGB transfer by table (the star's face, glare and events, drawn on the CPU each picture): the exact curve costs a
 * power per channel per pixel, which at the close-up's sizes was most of a picture's time. `srgb255` is the encoded
 * value times 255 for linear light (clamped to 0..1), from a 4096-step table read with linear interpolation (under a
 * hundredth of a step off the curve); `linear255` decodes a byte exactly (256 entries).
 */
const N = 4096;
const ENC = new Float64Array(N + 1);
for (let i = 0; i <= N; i++) {
  const c = i / N;
  ENC[i] = 255 * (c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055);
}
const DEC = new Float64Array(256);
for (let i = 0; i < 256; i++) {
  const c = i / 255;
  DEC[i] = c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
}

export function srgb255(v: number): number {
  if (!(v > 0)) return 0;
  if (v >= 1) return 255;
  const f = v * N;
  const i = Math.floor(f);
  return ENC[i]! + (f - i) * (ENC[i + 1]! - ENC[i]!);
}

/** Linear light of an sRGB byte. */
export function linear255(byte: number): number {
  return DEC[Math.max(0, Math.min(255, Math.round(byte)))]!;
}
