// The canvas tier of the relief, graticule and contour passes: one pass over
// the pixels of an orthographic globe, at the plan's render scale. No GPU.
import { fastFace, lightDir, lookupFast, viewRotation, type Prepared } from "./surface.ts";
import type { Plan } from "./recipe.ts";
import type { AirRender } from "./gpu-air.ts";

/** Kasten and Young (1989) relative air mass for a cosine of the zenith angle (cos > -0.08; else a large number). */
export function airMass(cosz: number): number {
  if (cosz < -0.08) return 1e4;
  const z = (Math.acos(Math.max(-1, Math.min(1, cosz))) * 180) / Math.PI;
  return 1 / (cosz + 0.50572 * Math.pow(Math.max(0.5, 96.07995 - z), -1.6364));
}

const num = (v: unknown, d: number) => (typeof v === "number" ? v : d);

/** A small body's lump (renderer small-body.ts `smallBodyShape`): its radius along a unit direction, at most 1. */
export type CanvasShape = { key: string; at: (d: [number, number, number]) => number };

const LUMP_W = 128;
const LUMP_H = 64;
const lumpTables = new Map<string, Float32Array>();

/** The lump's radius on a latitude-longitude table, read once per shape (the canvas asks it many times a pixel). */
function lumpTable(s: CanvasShape): Float32Array {
  const hit = lumpTables.get(s.key);
  if (hit) return hit;
  const t = new Float32Array((LUMP_W + 1) * (LUMP_H + 1));
  for (let j = 0; j <= LUMP_H; j++) {
    const la = -Math.PI / 2 + (Math.PI * j) / LUMP_H;
    for (let i = 0; i <= LUMP_W; i++) {
      const lo = -Math.PI + (2 * Math.PI * i) / LUMP_W;
      t[j * (LUMP_W + 1) + i] = s.at([Math.cos(la) * Math.cos(lo), Math.cos(la) * Math.sin(lo), Math.sin(la)]);
    }
  }
  if (lumpTables.size > 32) lumpTables.clear();
  lumpTables.set(s.key, t);
  return t;
}

/** The table's radius along a direction (any length), bilinear. */
function lumpAt(t: Float32Array, x: number, y: number, z: number): number {
  const l = Math.hypot(x, y, z) || 1;
  const la = Math.asin(Math.max(-1, Math.min(1, z / l)));
  const lo = Math.atan2(y, x);
  const u = ((lo + Math.PI) / (2 * Math.PI)) * LUMP_W;
  const v = ((la + Math.PI / 2) / Math.PI) * LUMP_H;
  const i = Math.min(LUMP_W - 1, Math.floor(u)), j = Math.min(LUMP_H - 1, Math.floor(v));
  const fu = u - i, fv = v - j;
  const a = t[j * (LUMP_W + 1) + i]!, b = t[j * (LUMP_W + 1) + i + 1]!;
  const c = t[(j + 1) * (LUMP_W + 1) + i]!, d = t[(j + 1) * (LUMP_W + 1) + i + 1]!;
  return (a * (1 - fu) + b * fu) * (1 - fv) + (c * (1 - fu) + d * fu) * fv;
}

/** Draw a plan into `out`, a 2D canvas `full` pixels square, rendering at the plan's scale and smoothing up. */
export function drawCanvas(out: HTMLCanvasElement, p: Prepared, plan: Plan, lon: number, lat: number, full: number, scratch: { c?: HTMLCanvasElement; img?: ImageData }, air?: AirRender | null, shape?: CanvasShape | null) {
  const n = Math.max(16, Math.round(full * plan.render_scale));
  const target = n === full ? out : (scratch.c ??= document.createElement("canvas"));
  if (target.width !== n) target.width = target.height = n;
  const cx = target.getContext("2d")!;
  const img = scratch.img && scratch.img.width === n ? scratch.img : (scratch.img = cx.createImageData(n, n));
  const data = img.data;
  data.fill(0);
  const relief = plan.passes.find((x) => x.pass === "relief-globe")?.params ?? {};
  const grat = plan.passes.find((x) => x.pass === "graticule")?.params;
  const cont = plan.passes.find((x) => x.pass === "contours")?.params;
  // Physical lighting has no canvas form; the nearest is full shading with a dark night side.
  const physical = relief.lighting === "physical";
  const shade = physical ? 1 : num(relief.shade, 0.8);
  const ambient = physical ? 0.03 : num(relief.ambient, 0.16);
  // Exposure: the GPU applies it before tone mapping; the canvas has no tone mapping, so it is a plain gain here.
  const gain = physical ? num(plan.passes.find((x) => x.pass === "atmosphere")?.params.exposure, 1) : 1;
  const L = lightDir(num(relief.sun_azimuth_deg, 315), num(relief.sun_elevation_deg, 40));
  const rot = viewRotation(lon, lat);
  const R = n / 2;
  // Contour bands and cover per pixel, for the second pass.
  const band = cont ? new Int32Array(n * n).fill(-0x7fffffff) : null;
  const wet = cont ? new Uint8Array(n * n) : null;
  const interval = num(cont?.interval_m, 1000);
  const gs = (num(grat?.spacing_deg, 30) * Math.PI) / 180;
  const ga = num(grat?.opacity, 0.3);
  const px = 1 / R; // one pixel in view units
  const [[r00, r01, r02], [r10, r11, r12], [r20, r21, r22]] = rot.rows;
  const flat = 1 - shade + shade * ambient;
  const dirK = shade * (1 - ambient);
  // A small body (RW16a): the same lump the GPU tiers draw, found along each pixel's view ray.
  const lump = shape ? lumpTable(shape) : null;
  const toWorld = (x: number, y: number, z: number): [number, number, number] => [r00 * x + r10 * y + r20 * z, r01 * x + r11 * y + r21 * z, r02 * x + r12 * y + r22 * z];
  for (let py = 0; py < n; py++) {
    const y = (R - py - 0.5) / R;
    for (let qx = 0; qx < n; qx++) {
      const x = (qx + 0.5 - R) / R;
      let r2 = x * x + y * y;
      if (r2 > (1 + 0.5 / R) * (1 + 0.5 / R)) continue;
      let rr2 = r2;
      if (r2 > 1) r2 = 1;
      let z = Math.sqrt(1 - r2);
      // The lump's surface along the ray: march in from the front of the unit sphere, then halve the step.
      let geo: [number, number, number] | null = null;
      if (lump) {
        const inside = (t: number) => {
          const [a, b, c] = toWorld(x, y, t);
          return Math.hypot(x, y, t) <= lumpAt(lump, a, b, c);
        };
        const STEPS = 24;
        let hit = NaN;
        for (let q = 0; q <= STEPS; q++) {
          const t = z - (2 * z * q) / STEPS;
          if (inside(t)) {
            let lo = t, hi = t + (2 * z) / STEPS;
            for (let b = 0; b < 8; b++) {
              const mid = (lo + hi) / 2;
              if (inside(mid)) lo = mid;
              else hi = mid;
            }
            hit = lo;
            break;
          }
        }
        if (Number.isNaN(hit)) continue;
        z = hit;
        rr2 = 0;
        // The lump's own normal from two neighbours on its surface.
        const w0 = toWorld(x, y, z);
        const l0 = Math.hypot(...w0);
        const u0: [number, number, number] = [w0[0] / l0, w0[1] / l0, w0[2] / l0];
        const t1 = Math.abs(u0[2]) < 0.9 ? [-u0[1], u0[0], 0] : [0, -u0[2], u0[1]];
        const t1l = Math.hypot(t1[0]!, t1[1]!, t1[2]!);
        const a1: [number, number, number] = [t1[0]! / t1l, t1[1]! / t1l, t1[2]! / t1l];
        const a2: [number, number, number] = [u0[1] * a1[2] - u0[2] * a1[1], u0[2] * a1[0] - u0[0] * a1[2], u0[0] * a1[1] - u0[1] * a1[0]];
        const e = 0.02;
        const at = (d: number[]) => {
          const rad = lumpAt(lump, d[0]!, d[1]!, d[2]!);
          const l = Math.hypot(d[0]!, d[1]!, d[2]!);
          return [(d[0]! / l) * rad, (d[1]! / l) * rad, (d[2]! / l) * rad];
        };
        const P0 = at(u0), P1 = at([u0[0] + e * a1[0], u0[1] + e * a1[1], u0[2] + e * a1[2]]), P2 = at([u0[0] + e * a2[0], u0[1] + e * a2[1], u0[2] + e * a2[2]]);
        const e1 = [P1[0]! - P0[0]!, P1[1]! - P0[1]!, P1[2]! - P0[2]!], e2 = [P2[0]! - P0[0]!, P2[1]! - P0[1]!, P2[2]! - P0[2]!];
        let gn: [number, number, number] = [e1[1]! * e2[2]! - e1[2]! * e2[1]!, e1[2]! * e2[0]! - e1[0]! * e2[2]!, e1[0]! * e2[1]! - e1[1]! * e2[0]!];
        if (gn[0] * u0[0] + gn[1] * u0[1] + gn[2] * u0[2] < 0) gn = [-gn[0], -gn[1], -gn[2]];
        const gl = Math.hypot(...gn) || 1;
        geo = [gn[0] / gl - u0[0], gn[1] / gl - u0[1], gn[2] / gl - u0[2]];
      }
      // View to world is the transpose of the rows.
      let dx = r00 * x + r10 * y + r20 * z;
      let dy = r01 * x + r11 * y + r21 * z;
      let dz = r02 * x + r12 * y + r22 * z;
      if (lump) {
        const l = Math.hypot(dx, dy, dz) || 1;
        dx /= l;
        dy /= l;
        dz /= l;
      }
      const k = lookupFast(p, dx, dy, dz);
      const o = (py * n + qx) * 4;
      if (fastFace < 0) {
        data[o] = data[o + 1] = data[o + 2] = 40;
        data[o + 3] = 255;
        continue;
      }
      const f = p.faces[fastFace]!;
      // On a lump, the tiles' own relief tilts the lump's normal (as the GPU tiers do).
      let nx = f.normal[k * 3]! + (geo ? geo[0] : 0);
      let ny = f.normal[k * 3 + 1]! + (geo ? geo[1] : 0);
      let nz = f.normal[k * 3 + 2]! + (geo ? geo[2] : 0);
      if (geo) {
        const l = Math.hypot(nx, ny, nz) || 1;
        nx /= l;
        ny /= l;
        nz /= l;
      }
      // World normal to view, dotted with the light.
      const ndl = (r00 * nx + r01 * ny + r02 * nz) * L[0] + (r10 * nx + r11 * ny + r12 * nz) * L[1] + (r20 * nx + r21 * ny + r22 * nz) * L[2];
      const g = f.cover[k] === 2 ? 1 : flat + dirK * (ndl > 0 ? ndl : 0);
      let r = f.rgb[k * 3]! * g;
      let gg = f.rgb[k * 3 + 1]! * g;
      let b = f.rgb[k * 3 + 2]! * g;
      if (grat) {
        // Distance to the nearest line, in view units, against about a pixel's width.
        const la = Math.asin(dz > 1 ? 1 : dz < -1 ? -1 : dz);
        const lo = Math.atan2(dy, dx);
        const w = (0.75 * px) / Math.max(z, 0.15);
        const dla = Math.abs(la - Math.round(la / gs) * gs);
        const dlo = Math.abs(lo - Math.round(lo / gs) * gs) * Math.cos(la);
        const eq = Math.abs(la) < w;
        if (dla < w || (dlo < w && Math.abs(la) < Math.PI / 2 - gs / 4)) {
          const a = Math.min(1, ga * (eq ? 1.6 : 1));
          r += (255 - r) * a;
          gg += (255 - gg) * a;
          b += (255 - b) * a;
        }
      }
      if (air && !lump) {
        // One-column approximation of the air (canvas tier): the ground dimmed and reddened by the
        // sun's path, then veiled by light scattered along the view's path, brightest where the sun is high.
        const cs = x * L[0] + y * L[1] + z * L[2];
        const amV = airMass(z);
        const amS = airMass(cs);
        const lightUp = Math.max(0, Math.min(1, cs * 1.2 + 0.1));
        const H = air.scale_height;
        const tv0 = Math.exp(-air.beta[0] * H * amV);
        const tv1 = Math.exp(-air.beta[1] * H * amV);
        const tv2 = Math.exp(-air.beta[2] * H * amV);
        r = r * Math.exp(-air.beta[0] * H * Math.max(0, amS - 1)) * tv0 + 255 * (1 - tv0) * lightUp * Math.exp(-air.beta[0] * H * amS);
        gg = gg * Math.exp(-air.beta[1] * H * Math.max(0, amS - 1)) * tv1 + 255 * (1 - tv1) * lightUp * Math.exp(-air.beta[1] * H * amS);
        b = b * Math.exp(-air.beta[2] * H * Math.max(0, amS - 1)) * tv2 + 255 * (1 - tv2) * lightUp * Math.exp(-air.beta[2] * H * amS);
      }
      data[o] = r * gain;
      data[o + 1] = gg * gain;
      data[o + 2] = b * gain;
      // Soften the rim: coverage of the pixel by the disk.
      const edge = (1 - Math.sqrt(rr2)) * R + 0.5;
      data[o + 3] = edge >= 1 ? 255 : 255 * edge;
      if (band) {
        const e = f.elev[k]!;
        band[py * n + qx] = f.cover[k] === 0 ? Math.floor(e / interval) : -0x7ffffff0;
        wet![py * n + qx] = f.cover[k] === 0 ? 0 : 1;
      }
    }
  }
  if (band && wet) {
    const a = num(cont?.opacity, 0.45);
    const shore = cont?.shoreline !== false;
    for (let py = 0; py < n - 1; py++) {
      for (let qx = 0; qx < n - 1; qx++) {
        const i = py * n + qx;
        if (band[i] === -0x7fffffff) continue;
        const right = i + 1;
        const below = i + n;
        if (band[right] === -0x7fffffff || band[below] === -0x7fffffff) continue;
        const coast = shore && (wet[i] !== wet[right] || wet[i] !== wet[below]);
        const line = !wet[i] && ((!wet[right] && band[i] !== band[right]) || (!wet[below] && band[i] !== band[below]));
        if (!coast && !line) continue;
        const k = coast ? Math.min(1, a * 1.8) : a;
        const o = i * 4;
        data[o] = data[o]! * (1 - k) + 30 * k;
        data[o + 1] = data[o + 1]! * (1 - k) + 34 * k;
        data[o + 2] = data[o + 2]! * (1 - k) + 40 * k;
      }
    }
  }
  cx.putImageData(img, 0, 0);
  if (target !== out) {
    if (out.width !== full) out.width = out.height = full;
    const o = out.getContext("2d")!;
    o.clearRect(0, 0, full, full);
    o.imageSmoothingEnabled = true;
    o.imageSmoothingQuality = "medium";
    o.drawImage(target, 0, 0, full, full);
  }
}
