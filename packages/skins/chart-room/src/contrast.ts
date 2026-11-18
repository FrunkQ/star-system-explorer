// WCAG 2.x contrast for the token pairs the schema declares.
function channel(c: number): number {
  const s = c / 255;
  return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
}

/** Parse #RRGGBB or rgba(r, g, b, a), composited over `under` when translucent. */
export function rgb(colour: string, under?: [number, number, number]): [number, number, number] {
  const hex = /^#([0-9a-f]{6})$/i.exec(colour);
  if (hex) {
    const n = parseInt(hex[1]!, 16);
    return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
  }
  const m = /^rgba?\(\s*(\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)$/.exec(colour);
  if (!m) throw new Error(`cannot read colour ${colour}`);
  const a = m[4] === undefined ? 1 : Number(m[4]);
  const c: [number, number, number] = [Number(m[1]), Number(m[2]), Number(m[3])];
  if (a >= 1) return c;
  if (!under) throw new Error(`translucent colour ${colour} needs a ground`);
  return c.map((v, i) => v * a + under[i]! * (1 - a)) as [number, number, number];
}

export function luminance([r, g, b]: [number, number, number]): number {
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

export function contrast(a: string, b: string): number {
  const gb = rgb(b);
  const la = luminance(rgb(a, gb));
  const lb = luminance(gb);
  return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
}
