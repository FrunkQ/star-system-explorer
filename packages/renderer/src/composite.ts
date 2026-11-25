// Composite (exotic objects design 6A, XO17): the pixel maths for laying several bands into one picture. Two forms of
// one rule, so a view can use whichever it has:
//
// - `composite(frames)`: for a view that holds its own frames (a float RGB buffer per band, as a GPU pass or a
//   canvas gives them). Each band is scaled to its own brightest, Visible kept in its natural colours, every other
//   band turned to its brightness and laid in its stated colour, then all are added and clipped.
// - `tintMatrix(colour, dark)`: the same tint as a 4×5 colour matrix (an SVG feColorMatrix), for the general
//   CompositeStack that lays whole view layers over each other in the browser (chart-grammar).
//
// Presentation only; nothing here is physics. Brightness weights are the sRGB luminance (Rec. 709) coefficients.

export const LUMA = [0.2126, 0.7152, 0.0722] as const;

/** A "#rrggbb" colour as 0..1 channels; "natural" or anything else is white. */
export function rgbOf(colour: string): [number, number, number] {
  const m = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(colour);
  if (!m) return [1, 1, 1];
  return [parseInt(m[1]!, 16) / 255, parseInt(m[2]!, 16) / 255, parseInt(m[3]!, 16) / 255];
}

/** One band's frame: RGB triples, any positive scale. */
export interface BandFrame {
  band: string;
  colour: string;
  rgb: Float32Array | number[];
}

/** Lay the frames into one: each scaled to its own brightest, tinted (Visible stays natural), added, clipped to 1. */
export function composite(frames: BandFrame[], pixels: number): Float32Array {
  const out = new Float32Array(pixels * 3);
  for (const f of frames) {
    let top = 0;
    for (let i = 0; i < pixels * 3; i++) top = Math.max(top, f.rgb[i] ?? 0);
    if (!(top > 0)) continue;
    const natural = f.band === "visible";
    const [cr, cg, cb] = rgbOf(f.colour);
    for (let p = 0; p < pixels; p++) {
      const r = (f.rgb[3 * p] ?? 0) / top;
      const g = (f.rgb[3 * p + 1] ?? 0) / top;
      const b = (f.rgb[3 * p + 2] ?? 0) / top;
      if (natural) {
        out[3 * p] = out[3 * p]! + r;
        out[3 * p + 1] = out[3 * p + 1]! + g;
        out[3 * p + 2] = out[3 * p + 2]! + b;
      } else {
        const y = LUMA[0] * r + LUMA[1] * g + LUMA[2] * b;
        out[3 * p] = out[3 * p]! + y * cr;
        out[3 * p + 1] = out[3 * p + 1]! + y * cg;
        out[3 * p + 2] = out[3 * p + 2]! + y * cb;
      }
    }
  }
  for (let i = 0; i < out.length; i++) out[i] = Math.min(1, out[i]!);
  return out;
}

/** The tint as an feColorMatrix (rows R, G, B, A; columns r, g, b, a, 1). On a dark ground light is laid in the
 * colour (black stays black, brightness becomes the colour); on a light ground ink is (white stays white, dark
 * marks take the colour). */
export function tintMatrix(colour: string, dark: boolean): number[] {
  const c = rgbOf(colour);
  const rows: number[] = [];
  for (let k = 0; k < 3; k++) {
    if (dark) rows.push(c[k]! * LUMA[0], c[k]! * LUMA[1], c[k]! * LUMA[2], 0, 0);
    else rows.push((1 - c[k]!) * LUMA[0], (1 - c[k]!) * LUMA[1], (1 - c[k]!) * LUMA[2], 0, c[k]!);
  }
  rows.push(0, 0, 0, 1, 0);
  return rows;
}

/** Brightness into every channel (for the ink tint). */
export const GREY: number[] = [...LUMA, 0, 0, ...LUMA, 0, 0, ...LUMA, 0, 0, 0, 0, 0, 1, 0];
/** How much the brightness is lifted so the chart room's paper (about 0.9 bright) reads as white and adds nothing. */
export const PAPER_LIFT = 1 / 0.88;
/** Grey in, ink out: white stays white, black becomes the colour. */
export function inkMatrix(colour: string): number[] {
  const c = rgbOf(colour);
  const rows: number[] = [];
  for (let k = 0; k < 3; k++) rows.push(1 - c[k]!, 0, 0, 0, c[k]!);
  rows.push(0, 0, 0, 1, 0);
  return rows;
}
