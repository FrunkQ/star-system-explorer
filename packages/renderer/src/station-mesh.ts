// The 3D model of a station (station-design.ts), built once from its parts: one plain triangle buffer per surface
// kind, texture coordinates in metres over the skin's panel size so plating is the same real size on a 30-person
// ring and a 20-million one. Pure data; three.js only in gpu-station.ts.
import type { Axis, Part, StationLayout, StationSurface } from "./station-design.ts";
import type { SurfaceBuffers } from "./ship-mesh.ts";

export interface StationMesh {
  surfaces: Partial<Record<StationSurface, SurfaceBuffers>>;
  triangles: number;
  bounds: [[number, number, number], [number, number, number]];
  /** The same without the solar and radiator arrays, for closing in on the station itself. */
  core_bounds: [[number, number, number], [number, number, number]];
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

/** A part's local frame: w along its axis, u and v across it (right-handed). */
function frame(axis: Axis): [[number, number, number], [number, number, number], [number, number, number]] {
  if (axis === "x") return [[0, 1, 0], [0, 0, 1], [1, 0, 0]];
  if (axis === "y") return [[0, 0, 1], [1, 0, 0], [0, 1, 0]];
  return [[1, 0, 0], [0, 1, 0], [0, 0, 1]];
}

/** Place local (a, b, w) round the part's centre. */
function at(p: Part, a: number, b: number, w: number): [number, number, number] {
  const [u, v, ax] = frame(p.axis);
  return [p.c[0] + a * u[0] + b * v[0] + w * ax[0], p.c[1] + a * u[1] + b * v[1] + w * ax[1], p.c[2] + a * u[2] + b * v[2] + w * ax[2]];
}
function dir(p: Part, a: number, b: number, w: number): [number, number, number] {
  const [u, v, ax] = frame(p.axis);
  return [a * u[0] + b * v[0] + w * ax[0], a * u[1] + b * v[1] + w * ax[1], a * u[2] + b * v[2] + w * ax[2]];
}

/** A lumpy rock's radius at a direction: a few seeded low harmonics, the same for every device. */
function lump(k: number, x: number, y: number, z: number): number {
  return 1 + k * (0.5 * Math.sin(3.1 * x + 1.3) * Math.cos(2.3 * y - 0.4) + 0.3 * Math.sin(5.2 * z + 2.1 * x) + 0.2 * Math.cos(7.7 * y + 3.3 * z));
}

function tube(b: Builder, p: Part, seg: number, panel: number): void {
  const L = p.len ?? 0;
  const r0 = p.r0 ?? 0;
  const r1 = p.r1 ?? r0;
  const slope = (r0 - r1) / Math.max(L, 1e-9);
  const k = 1 / Math.sqrt(1 + slope * slope);
  const base = b.p.length / 3;
  const rMax = Math.max(r0, r1);
  for (let s = 0; s <= seg; s++) {
    const t = (2 * Math.PI * s) / seg;
    const c = Math.cos(t);
    const sn = Math.sin(t);
    const n = dir(p, c * k, sn * k, slope * k);
    b.vert(...at(p, r0 * c, r0 * sn, -L / 2), ...n, 0, (t * rMax) / panel);
    b.vert(...at(p, r1 * c, r1 * sn, L / 2), ...n, L / panel, (t * rMax) / panel);
  }
  for (let s = 0; s < seg; s++) {
    const a = base + 2 * s;
    b.i.push(a, a + 2, a + 1, a + 1, a + 2, a + 3);
  }
  for (const [w, r, sgn] of [[-L / 2, r0, -1], [L / 2, r1, 1]] as const) {
    if (r <= 0) continue;
    const n = dir(p, 0, 0, sgn);
    const centre = b.vert(...at(p, 0, 0, w), ...n, 0, 0);
    const ring = b.p.length / 3;
    for (let s = 0; s <= seg; s++) {
      const t = (2 * Math.PI * s) / seg;
      b.vert(...at(p, r * Math.cos(t), r * Math.sin(t), w), ...n, (r * Math.cos(t)) / panel, (r * Math.sin(t)) / panel);
    }
    for (let s = 0; s < seg; s++) sgn > 0 ? b.i.push(centre, ring + s, ring + s + 1) : b.i.push(centre, ring + s + 1, ring + s);
  }
}

function torus(b: Builder, p: Part, seg: number, panel: number): void {
  const R = p.R ?? 0;
  const r = p.r ?? 0;
  // Enough segments round the big circle that a 75 km ring is still round.
  const big = Math.max(seg, Math.min(256, seg * 4));
  const small = Math.max(8, Math.round(seg / 2));
  const base = b.p.length / 3;
  for (let i = 0; i <= big; i++) {
    const t = (2 * Math.PI * i) / big;
    for (let j = 0; j <= small; j++) {
      const q = (2 * Math.PI * j) / small;
      const rr = R + r * Math.cos(q);
      const n = dir(p, Math.cos(q) * Math.cos(t), Math.cos(q) * Math.sin(t), Math.sin(q));
      b.vert(...at(p, rr * Math.cos(t), rr * Math.sin(t), r * Math.sin(q)), ...n, (t * R) / panel, (q * r) / panel);
    }
  }
  const row = small + 1;
  for (let i = 0; i < big; i++)
    for (let j = 0; j < small; j++) {
      const a = base + i * row + j;
      b.i.push(a, a + row, a + 1, a + 1, a + row, a + row + 1);
    }
}

function sphere(b: Builder, p: Part, seg: number, panel: number): void {
  const r = p.r ?? 0;
  const lumpy = p.lumpy ?? 0;
  const rings = Math.max(6, Math.round(seg / 2));
  const lat0 = p.hemi ? 0 : -Math.PI / 2;
  const base = b.p.length / 3;
  for (let i = 0; i <= rings; i++) {
    const la = lat0 + ((Math.PI / 2 - lat0) * i) / rings;
    for (let j = 0; j <= seg; j++) {
      const lo = (2 * Math.PI * j) / seg;
      const x = Math.cos(la) * Math.cos(lo);
      const y = Math.cos(la) * Math.sin(lo);
      const z = Math.sin(la);
      const k = lumpy ? lump(lumpy, x, y, z) : 1;
      // The normal of a lumpy rock is the sphere's: the roughness reads in the outline, not the shading.
      b.vert(...at(p, r * k * x, r * k * y, r * k * z), ...dir(p, x, y, z), (lo * r) / panel, (la * r) / panel);
    }
  }
  const row = seg + 1;
  for (let i = 0; i < rings; i++)
    for (let j = 0; j < seg; j++) {
      const a = base + i * row + j;
      b.i.push(a, a + 1, a + row, a + 1, a + row + 1, a + row);
    }
  if (p.hemi) {
    // A floor under a dome.
    const n = dir(p, 0, 0, -1);
    const centre = b.vert(...at(p, 0, 0, 0), ...n, 0, 0);
    const ring = b.p.length / 3;
    for (let j = 0; j <= seg; j++) {
      const lo = (2 * Math.PI * j) / seg;
      b.vert(...at(p, r * Math.cos(lo), r * Math.sin(lo), 0), ...n, 0, 0);
    }
    for (let j = 0; j < seg; j++) b.i.push(centre, ring + j + 1, ring + j);
  }
}

function box(b: Builder, p: Part, panel: number): void {
  const [sx, sy, sz] = p.size ?? [1, 1, 1];
  const h = [sx / 2, sy / 2, sz / 2];
  const faces: [number, number, number][] = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];
  for (const n of faces) {
    const k = n.findIndex((v) => v !== 0);
    const a = (k + 1) % 3;
    const c = (k + 2) % 3;
    const sgn = n[k]!;
    const base = b.p.length / 3;
    for (const [da, dc] of [[-1, -1], [1, -1], [1, 1], [-1, 1]] as const) {
      const q = [0, 0, 0];
      q[k] = sgn * h[k]!;
      q[a] = da * h[a]!;
      q[c] = dc * h[c]!;
      b.vert(p.c[0] + q[0]!, p.c[1] + q[1]!, p.c[2] + q[2]!, ...n, (q[a]! + h[a]!) / panel, (q[c]! + h[c]!) / panel);
    }
    sgn > 0 ? b.i.push(base, base + 1, base + 2, base, base + 2, base + 3) : b.i.push(base, base + 2, base + 1, base, base + 3, base + 2);
  }
}

function panelPart(b: Builder, p: Part, t: number, panel: number): void {
  const [A, B] = p.size ?? [1, 1, 0];
  for (const face of [1, -1]) {
    const n = dir(p, 0, 0, face);
    const q = ([[-1, -1], [1, -1], [1, 1], [-1, 1]] as const).map(([a, c]) => b.vert(...at(p, (a * A) / 2, (c * B) / 2, face * t), ...n, ((a + 1) * A) / 2 / panel, ((c + 1) * B) / 2 / panel));
    face > 0 ? b.i.push(q[0]!, q[1]!, q[2]!, q[0]!, q[2]!, q[3]!) : b.i.push(q[0]!, q[2]!, q[1]!, q[0]!, q[3]!, q[2]!);
  }
}

/** Build the buffers. `segments` round each curved part comes from the device tier. */
export function stationMesh(l: StationLayout, o: { segments: number; panel_m: number }): StationMesh {
  const builders = new Map<StationSurface, Builder>();
  const lo: [number, number, number] = [Infinity, Infinity, Infinity];
  const hi: [number, number, number] = [-Infinity, -Infinity, -Infinity];
  const clo: [number, number, number] = [Infinity, Infinity, Infinity];
  const chi: [number, number, number] = [-Infinity, -Infinity, -Infinity];
  const thin = Math.max(0.05, 0.002 * l.size_m);
  for (const p of l.parts) {
    const b = builders.get(p.surface) ?? (builders.set(p.surface, new Builder()), builders.get(p.surface)!);
    const start = b.p.length;
    if (p.form === "tube") tube(b, p, o.segments, o.panel_m);
    else if (p.form === "torus") torus(b, p, o.segments, o.panel_m);
    else if (p.form === "sphere") sphere(b, p, o.segments, o.panel_m);
    else if (p.form === "box") box(b, p, o.panel_m);
    else panelPart(b, p, thin, o.panel_m);
    const arrays = !(p.array || p.surface === "solar" || p.surface === "radiators");
    for (let k = start; k < b.p.length; k += 3)
      for (let d = 0; d < 3; d++) {
        const v = b.p[k + d]!;
        (lo[d] = Math.min(lo[d]!, v)), (hi[d] = Math.max(hi[d]!, v));
        if (arrays) (clo[d] = Math.min(clo[d]!, v)), (chi[d] = Math.max(chi[d]!, v));
      }
  }
  const surfaces: StationMesh["surfaces"] = {};
  let triangles = 0;
  for (const [k, b] of builders) {
    surfaces[k] = b.done();
    triangles += b.i.length / 3;
  }
  return { surfaces, triangles, bounds: [lo, hi], core_bounds: Number.isFinite(clo[0]) ? [clo, chi] : [lo, hi] };
}
