// Colour vision (V3 gap item 30, Alex 23 Nov 2026: no third "clarity" skin; every look is checked instead).
// Two colours that must be told apart (the accent from the ink, field out from field in, one element origin from
// another) are compared as a person with each common colour vision deficiency sees them, at full severity:
// protanopia, deuteranopia and tritanopia, by the Machado, Oliveira and Fernandes (2009) model, and also as
// everyone sees them. The difference is CIE76 ΔE in CIELAB (D65), which counts lightness as well as hue, so two
// colours of one hue told apart by lightness pass.
//
// Machado, G. M., Oliveira, M. M. and Fernandes, L. A. F. (2009), "A physiologically-based model for simulation of
// color vision deficiency", IEEE TVCG 15(6), 1291-1298, doi:10.1109/TVCG.2009.113 (matrices for severity 1.0).
import { rgb } from "./contrast.ts";

export type Vision = "typical" | "protanopia" | "deuteranopia" | "tritanopia";
export const VISIONS: readonly Vision[] = ["typical", "protanopia", "deuteranopia", "tritanopia"];

const MATRIX: Record<Exclude<Vision, "typical">, readonly number[]> = {
  protanopia: [0.152286, 1.052583, -0.204868, 0.114503, 0.786281, 0.099216, -0.003882, -0.048116, 1.051998],
  deuteranopia: [0.367322, 0.860646, -0.227968, 0.280085, 0.672501, 0.047413, -0.01182, 0.04294, 0.968881],
  tritanopia: [1.255528, -0.076749, -0.178779, -0.078411, 0.930809, 0.147602, 0.004733, 0.691367, 0.3039],
};

const toLinear = (c: number) => {
  const s = c / 255;
  return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
};

/** A colour as seen with the given vision, in linear sRGB (0..1 each). */
export function seen(colour: string, vision: Vision, under?: string): [number, number, number] {
  const lin = rgb(colour, under ? rgb(under) : undefined).map(toLinear) as [number, number, number];
  if (vision === "typical") return lin;
  const m = MATRIX[vision];
  const clamp = (v: number) => Math.min(1, Math.max(0, v));
  return [0, 1, 2].map((i) => clamp(m[i * 3]! * lin[0] + m[i * 3 + 1]! * lin[1] + m[i * 3 + 2]! * lin[2])) as [number, number, number];
}

/** Linear sRGB to CIELAB (D65). */
export function lab([r, g, b]: [number, number, number]): [number, number, number] {
  const x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047;
  const y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  const z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883;
  const f = (t: number) => (t > 216 / 24389 ? Math.cbrt(t) : (24389 / 27 * t + 16) / 116);
  return [116 * f(y) - 16, 500 * (f(x) - f(y)), 200 * (f(y) - f(z))];
}

/** How far apart two colours look with the given vision (CIE76 ΔE). Translucent colours sit on `under`. */
export function apart(a: string, b: string, vision: Vision, under?: string): number {
  const p = lab(seen(a, vision, under));
  const q = lab(seen(b, vision, under));
  return Math.hypot(p[0] - q[0], p[1] - q[1], p[2] - q[2]);
}

/** The least difference between two colours over every vision. */
export function leastApart(a: string, b: string, under?: string): { delta: number; vision: Vision } {
  return VISIONS.map((vision) => ({ delta: apart(a, b, vision, under), vision })).reduce((m, x) => (x.delta < m.delta ? x : m));
}
