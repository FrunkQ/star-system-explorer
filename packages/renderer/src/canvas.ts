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

/** Draw a plan into `out`, a 2D canvas `full` pixels square, rendering at the plan's scale and smoothing up. */
export function drawCanvas(out: HTMLCanvasElement, p: Prepared, plan: Plan, lon: number, lat: number, full: number, scratch: { c?: HTMLCanvasElement; img?: ImageData }, air?: AirRender | null) {
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
  for (let py = 0; py < n; py++) {
    const y = (R - py - 0.5) / R;
    for (let qx = 0; qx < n; qx++) {
      const x = (qx + 0.5 - R) / R;
      let r2 = x * x + y * y;
      if (r2 > (1 + 0.5 / R) * (1 + 0.5 / R)) continue;
      const rr2 = r2;
      if (r2 > 1) r2 = 1;
      const z = Math.sqrt(1 - r2);
      // View to world is the transpose of the rows.
      const dx = r00 * x + r10 * y + r20 * z;
      const dy = r01 * x + r11 * y + r21 * z;
      const dz = r02 * x + r12 * y + r22 * z;
      const k = lookupFast(p, dx, dy, dz);
      const o = (py * n + qx) * 4;
      if (fastFace < 0) {
        data[o] = data[o + 1] = data[o + 2] = 40;
        data[o + 3] = 255;
        continue;
      }
      const f = p.faces[fastFace]!;
      const nx = f.normal[k * 3]!;
      const ny = f.normal[k * 3 + 1]!;
      const nz = f.normal[k * 3 + 2]!;
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
      if (air) {
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
      data[o] = r;
      data[o + 1] = gg;
      data[o + 2] = b;
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
