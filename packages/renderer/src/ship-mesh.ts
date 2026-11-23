// V3 gap item 14, stage 3: the 3D model, built once from the same layout the side and top views draw, so the model
// and the plans always agree. Generation is a one-off and may be heavy; the result is a handful of plain triangle
// buffers (one per surface kind) that any device draws cheaply. Texture coordinates run in metres over the style's
// panel size, so panels are the same real size on a 30 m courier and a 5 km torch. Pure data here; three.js only in
// gpu-ship.ts.
import type { Piece, ShipLayout, SystemKind } from "./ship-design.ts";

/** Surface kinds: each gets one buffer and one material from the style's `skin`. */
export type Surface = "hull" | "tanks" | "radiators" | "spine" | "drive" | "shield";

export const SURFACE_OF: Record<SystemKind, Surface | null> = {
  sensors: "hull",
  crew: "hull",
  cargo: "hull",
  tanks: "tanks",
  radiators: "radiators",
  drive: "drive",
  spine: "spine",
  plume: null,
  shield: "shield",
  wings: "hull",
  legs: "spine",
  chute: "hull",
  scoop: "drive",
};

export interface SurfaceBuffers {
  positions: Float32Array;
  normals: Float32Array;
  /** Metres along and around, over the style's panel size. */
  uvs: Float32Array;
  indices: Uint32Array;
}

export interface ShipMesh {
  surfaces: Partial<Record<Surface, SurfaceBuffers>>;
  triangles: number;
  /** [min, max] corners in metres, x along the ship from the stern. */
  bounds: [[number, number, number], [number, number, number]];
  /** The same for the modules alone (no radiators, no spine), for closing in on the ship itself. */
  hull_bounds: [[number, number, number], [number, number, number]];
  /** The modules without the drive either, for an aerobraking pass, where the shield end is what matters. */
  front_bounds: [[number, number, number], [number, number, number]];
}

class Builder {
  p: number[] = [];
  n: number[] = [];
  uv: number[] = [];
  i: number[] = [];
  vert(x: number, y: number, z: number, nx: number, ny: number, nz: number, u: number, v: number): number {
    this.p.push(x, y, z);
    this.n.push(nx, ny, nz);
    this.uv.push(u, v);
    return this.p.length / 3 - 1;
  }
  done(): SurfaceBuffers {
    return { positions: new Float32Array(this.p), normals: new Float32Array(this.n), uvs: new Float32Array(this.uv), indices: new Uint32Array(this.i) };
  }
}

/** A frustum about a line parallel to x through (y, z), radius r0 at x0 and r1 at x1, with flat ends. */
function body(b: Builder, p: Piece, segments: number, panel: number): void {
  const len = p.x1 - p.x0;
  // The side's normal tilts by the slope of the frustum.
  const slope = (p.r0 - p.r1) / Math.max(len, 1e-9);
  const k = 1 / Math.sqrt(1 + slope * slope);
  const base = b.p.length / 3;
  const rMax = Math.max(p.r0, p.r1);
  for (let s = 0; s <= segments; s++) {
    const a = (2 * Math.PI * s) / segments;
    const c = Math.cos(a);
    const sn = Math.sin(a);
    const around = (a * rMax) / panel;
    b.vert(p.x0, p.y + p.r0 * c, p.z + p.r0 * sn, slope * k, c * k, sn * k, 0, around);
    b.vert(p.x1, p.y + p.r1 * c, p.z + p.r1 * sn, slope * k, c * k, sn * k, len / panel, around);
  }
  for (let s = 0; s < segments; s++) {
    const a = base + 2 * s;
    b.i.push(a, a + 2, a + 1, a + 1, a + 2, a + 3);
  }
  // End caps (a radius of zero needs none).
  for (const [x, r, nx] of [[p.x0, p.r0, -1], [p.x1, p.r1, 1]] as const) {
    if (r <= 0) continue;
    const centre = b.vert(x, p.y, p.z, nx, 0, 0, 0, 0);
    const ring = b.p.length / 3;
    for (let s = 0; s <= segments; s++) {
      const a = (2 * Math.PI * s) / segments;
      b.vert(x, p.y + r * Math.cos(a), p.z + r * Math.sin(a), nx, 0, 0, (r * Math.cos(a)) / panel, (r * Math.sin(a)) / panel);
    }
    for (let s = 0; s < segments; s++) nx > 0 ? b.i.push(centre, ring + s, ring + s + 1) : b.i.push(centre, ring + s + 1, ring + s);
  }
}

/** A flat panel in the x-y plane, root from x0 to x1 at y, reaching `span` out with its tip swept back, `t` thick. */
function panelPiece(b: Builder, p: Piece, t: number, panel: number): void {
  const s = p.span ?? 0;
  const sw = p.sweep ?? 0;
  const corners: [number, number][] = [[p.x0, p.y], [p.x1, p.y], [p.x1 - sw, p.y + s], [p.x0 - sw, p.y + s]];
  for (const face of [1, -1]) {
    const q = corners.map(([x, y]) => b.vert(x, y, p.z + face * t, 0, 0, face, x / panel, y / panel));
    // Keep both faces wound outward whichever way the panel reaches.
    const flip = face * Math.sign(s || 1) < 0;
    flip ? b.i.push(q[0]!, q[2]!, q[1]!, q[0]!, q[3]!, q[2]!) : b.i.push(q[0]!, q[1]!, q[2]!, q[0]!, q[2]!, q[3]!);
  }
}

/** Build the model's buffers from a layout. `segments` round each body (a device tier sets it); `panel_m` is the
 * style's panel size. */
export function shipMesh(l: ShipLayout, o: { segments: number; panel_m: number }): ShipMesh {
  const builders = new Map<Surface, Builder>();
  const big = Math.max(...l.pieces.filter((p) => p.shape === "body").map((p) => Math.max(p.r0, p.r1)));
  const lo: [number, number, number] = [Infinity, Infinity, Infinity];
  const hi: [number, number, number] = [-Infinity, -Infinity, -Infinity];
  const hlo: [number, number, number] = [Infinity, Infinity, Infinity];
  const hhi: [number, number, number] = [-Infinity, -Infinity, -Infinity];
  const flo: [number, number, number] = [Infinity, Infinity, Infinity];
  const fhi: [number, number, number] = [-Infinity, -Infinity, -Infinity];
  for (const p of l.pieces) {
    const surface = SURFACE_OF[p.system];
    if (!surface) continue;
    const b = builders.get(surface) ?? (builders.set(surface, new Builder()), builders.get(surface)!);
    const start = b.p.length;
    if (p.shape === "body") body(b, p, o.segments, o.panel_m);
    else panelPiece(b, p, Math.max(0.05, 0.01 * big), o.panel_m);
    for (let k = start; k < b.p.length; k += 3)
      for (let d = 0; d < 3; d++) {
        const v = b.p[k + d]!;
        (lo[d] = Math.min(lo[d]!, v)), (hi[d] = Math.max(hi[d]!, v));
        if (surface !== "radiators" && surface !== "spine") (hlo[d] = Math.min(hlo[d]!, v)), (hhi[d] = Math.max(hhi[d]!, v));
        if (surface !== "radiators" && surface !== "spine" && surface !== "drive") (flo[d] = Math.min(flo[d]!, v)), (fhi[d] = Math.max(fhi[d]!, v));
      }
  }
  const surfaces: Partial<Record<Surface, SurfaceBuffers>> = {};
  let triangles = 0;
  for (const [k, b] of builders) {
    surfaces[k] = b.done();
    triangles += b.i.length / 3;
  }
  return { surfaces, triangles, bounds: [lo, hi], hull_bounds: [hlo, hhi], front_bounds: [flo, fhi] };
}

export interface SurfaceLook {
  albedo: [number, number, number];
  roughness: number;
  metalness: number;
}

/** base-ships style `skin`: per surface, a linear albedo, roughness and metalness, and the panel look. */
export interface SkinSpec {
  panel_m: number;
  /** How much panels differ in shade (0 to 1). */
  patchiness: number;
  /** Fine scuffing (0 to 1). */
  wear: number;
  /** A style may leave out `shield`: it then takes the hull's look. */
  surfaces: Record<Exclude<Surface, "shield">, SurfaceLook> & { shield?: SurfaceLook };
}

/** A tileable panel texture, `px` square, covering 4 × 4 panels: each panel its own shade, dark seams between
 * them, and fine wear. Values are a brightness factor around 1 (multiply the albedo), so one texture serves every
 * surface. The same seed always gives the same texture. */
export function panelTexture(seed: number, px: number, skin: Pick<SkinSpec, "patchiness" | "wear">): Uint8Array {
  const out = new Uint8Array(px * px * 4);
  const cell = px / 4;
  const hash = (a: number, b: number) => {
    let h = (Math.imul(a, 374761393) + Math.imul(b, 668265263) + Math.imul(seed, 2246822519)) >>> 0;
    h = Math.imul(h ^ (h >>> 13), 1274126177) >>> 0;
    return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
  };
  for (let y = 0; y < px; y++)
    for (let x = 0; x < px; x++) {
      const cx = Math.floor(x / cell);
      const cy = Math.floor(y / cell);
      let v = 1 + skin.patchiness * (hash(cx, cy) - 0.5) * 0.5;
      // Seams: a pixel at a panel's edge.
      const ex = x % cell;
      const ey = y % cell;
      if (ex === 0 || ey === 0) v *= 0.55;
      v *= 1 + skin.wear * (hash(x + 1000, y) - 0.5) * 0.3;
      const byte = Math.max(0, Math.min(255, Math.round(v * 200)));
      const i = 4 * (y * px + x);
      out[i] = out[i + 1] = out[i + 2] = byte;
      out[i + 3] = 255;
    }
  return out;
}

/** Brightness factor a texel byte stands for (200 is 1). */
export const PANEL_UNIT = 200;
