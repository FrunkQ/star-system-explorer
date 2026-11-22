// The Window (V8): the pure parts. The kernel's `window_view` (sse-nav sky)
// says where each object is in the sky, how big and how bright, and whether
// the sensor sees it; this file only points a camera (a gnomonic projection,
// geometry) and puts words to the numbers. No physics here.

export interface WindowAsk {
  nav?: import("./route.ts").NavSystem;
  fixture?: string;
  at_s?: number;
  from?: string;
  at_au?: [number, number, number];
  sensor?: string;
  /** The system's address, so the kernel can say how an inline nav's system sits in the galaxy (`frame`). */
  system?: string;
}

export interface Seen {
  address: string;
  name: string;
  kind: "star" | "planet" | string;
  lon_deg: number;
  lat_deg: number;
  distance_au: number;
  angular_diameter_deg: number;
  magnitude: number;
  phase_deg: number;
  lit_fraction: number;
  detected: boolean;
  resolved: boolean;
  light_s: number;
  why: string;
}

export interface WindowView {
  at_s: number;
  viewer: [number, number, number];
  sensor: { name: string; limit_mag: number; resolution_arcsec: number; field_deg: number; band: string; why: string };
  sensor_id: string;
  beside: string | null;
  seen: Seen[];
  missed: number;
  model: string[];
  /** How the system's own frame sits in the galaxy (kernel, `frame.rs`): null when the kernel cannot say. */
  frame?: {
    pole_l_deg: number;
    pole_b_deg: number;
    zero_l_deg?: number;
    zero_b_deg?: number;
    node_l_deg?: number;
    to_galactic: [[number, number, number], [number, number, number], [number, number, number]];
    evidence: string;
    source?: string;
  } | null;
}

export const SENSORS: readonly { id: string; label: string }[] = [
  { id: "eye", label: "Eyes" },
  { id: "binoculars", label: "Binoculars" },
  { id: "telescope", label: "Telescope" },
];

type V3 = [number, number, number];
const unit = (lon: number, lat: number): V3 => {
  const l = (lon * Math.PI) / 180;
  const b = (lat * Math.PI) / 180;
  return [Math.cos(b) * Math.cos(l), Math.cos(b) * Math.sin(l), Math.sin(b)];
};
const dot = (a: V3, b: V3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const cross = (a: V3, b: V3): V3 => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const norm = (a: V3): V3 => {
  const n = Math.hypot(...a) || 1;
  return [a[0] / n, a[1] / n, a[2] / n];
};

/** A camera looking at (lon, lat) with a square field `field_deg` across, `size` pixels a side, ecliptic north up. */
export interface Camera { f: V3; r: V3; u: V3; scale: number; half: number; size: number; field_deg: number }
export function camera(lon_deg: number, lat_deg: number, field_deg: number, size: number): Camera {
  const f = unit(lon_deg, Math.max(-89.9, Math.min(89.9, lat_deg)));
  const r = norm(cross(f, [0, 0, 1]));
  const u = cross(r, f);
  const half = Math.tan(((Math.min(170, field_deg) / 2) * Math.PI) / 180);
  return { f, r, u, scale: size / 2 / half, half, size, field_deg };
}

/** Where a sky direction lands in the picture (null if behind or outside). */
export function project(c: Camera, lon_deg: number, lat_deg: number, margin = 0): { x: number; y: number } | null {
  const v = unit(lon_deg, lat_deg);
  const z = dot(v, c.f);
  if (z <= 1e-6) return null;
  const x = dot(v, c.r) / z;
  const y = dot(v, c.u) / z;
  const lim = c.half + margin / c.scale;
  if (Math.abs(x) > lim || Math.abs(y) > lim) return null;
  return { x: c.size / 2 + x * c.scale, y: c.size / 2 - y * c.scale };
}

/** The sky direction (a unit vector in the camera's frame) under a picture point: the inverse of `project`. */
export function unprojectCamera(c: Camera, x: number, y: number): V3 {
  const a = (x - c.size / 2) / c.scale;
  const b = (c.size / 2 - y) / c.scale;
  return norm([c.f[0] + a * c.r[0] + b * c.u[0], c.f[1] + a * c.r[1] + b * c.u[1], c.f[2] + a * c.r[2] + b * c.u[2]]);
}

/** A disc's radius in pixels at the picture's centre scale. */
export function discPx(c: Camera, angular_diameter_deg: number): number {
  return (Math.tan(((angular_diameter_deg / 2) * Math.PI) / 180) * c.scale);
}

/** A point's radius by brightness: brighter is bigger, the faintest seen is a dot. */
export function pointPx(magnitude: number, limit: number): number {
  return Math.max(0.8, Math.min(6, 0.8 + (limit - magnitude) * 0.45));
}

export function magWords(mag: number): string {
  if (mag < -20) return "blinding";
  if (mag < -4) return "brighter than anything in Earth's night sky but the Moon";
  if (mag < 0) return "among the brightest lights in the sky";
  if (mag < 3) return "an easy naked-eye light";
  if (mag < 6) return "faint to the eye";
  if (mag < 10) return "needs binoculars";
  if (mag < 18) return "needs a telescope";
  return "beyond the ship's telescope";
}

export function angleWords(deg: number): string {
  if (deg >= 1) return `${deg.toFixed(deg < 10 ? 1 : 0)}°`;
  const arcmin = deg * 60;
  if (arcmin >= 1) return `${arcmin.toFixed(arcmin < 10 ? 1 : 0)}′`;
  const arcsec = arcmin * 60;
  return `${arcsec < 0.1 ? arcsec.toExponential(1) : arcsec.toFixed(arcsec < 10 ? 2 : 0)}″`;
}

/** The screen angle (radians, canvas y down) from a point toward a sky direction, even one outside the picture. */
export function towards(c: Camera, from: { x: number; y: number }, lon_deg: number, lat_deg: number): number {
  const p = project(c, lon_deg, lat_deg, 1e9);
  if (p) return Math.atan2(p.y - from.y, p.x - from.x);
  const v = unit(lon_deg, lat_deg);
  return Math.atan2(-dot(v, c.u), dot(v, c.r));
}
