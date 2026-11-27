// Close-up (Alex, 25 Nov: "the UI to be able to get in close for beauty shots"): the camera in True may come down to
// a low, grazing orbit over the world at the centre of the frame. Pure numbers, shared by every view that orbits a
// world: how close it may come, how a zoom step moves near the ground (by height, not by distance from the centre),
// where it looks (from the centre at altitude, tipping toward the horizon as it comes down), and the ground under
// it for the detail ladder's deep tiles (VP3 part 4 reads `GroundView`).
import type { Vec3 } from "./frames.ts";

/** The lowest the camera comes, metres over the ground (a high aircraft's height): a few km in every world. */
export const CLOSE_FLOOR_M = 3000;
/** ...but never under this share of the radius, so a giant keeps a little of its curve in view (14 km over Jupiter). */
export const CLOSE_FLOOR_SHARE = 2e-4;
/**
 * Over a star the camera stops this share of the radius above the surface (the star close-up): 2.6 radii from its
 * centre the whole disc just fills a 45-degree view (asin(1/2.6) is 22.6 degrees), limb and prominences in sight. The
 * star's face and glow are flat pictures facing the camera (gpu.ts sprites) at most 512 and 640 px across; nearer,
 * they would be blown up past their detail and lose the sphere's shape (fudge register).
 */
export const STAR_CLOSE_SHARE = 1.6;
/** Within this many radii of the centre a zoom step scales the height over the ground, and the view starts to tip. */
export const CLOSE_REACH = 4;

/** The nearest the camera's distance from the world's centre may come, in the drawing's units (`r` its drawn radius). */
export function closeMin(r: number, radius_m?: number | null, star = false): number {
  if (star) return r * (1 + STAR_CLOSE_SHARE);
  const share = radius_m && radius_m > 0 ? Math.max(CLOSE_FLOOR_SHARE, CLOSE_FLOOR_M / radius_m) : 0.01;
  return r * (1 + share);
}

/**
 * The radius of a sphere's outline as a flat picture at its centre facing the camera: a sphere of radius `r` seen from
 * `d` away fills asin(r/d) of the view, which at its centre's distance is r/sqrt(1 - (r/d)^2), more than r up close.
 */
export function outlineRadius(r: number, d: number): number {
  const q = d > 0 ? Math.min(0.999, r / d) : 0;
  return r / Math.sqrt(1 - q * q);
}

/** One zoom step by `f` (below 1 nearer): far out it scales the distance, near the world the height over the ground. */
export function zoomNear(dist: number, r: number, f: number): number {
  return r > 0 && dist < CLOSE_REACH * r ? r + (dist - r) * f : dist * f;
}

const norm = (v: Vec3): Vec3 => {
  const n = Math.hypot(v[0], v[1], v[2]) || 1;
  return [v[0] / n, v[1] / n, v[2] / n];
};
const dot = (a: Vec3, b: Vec3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];

/**
 * Where the camera looks from `eye` at a world of drawn radius `r` centred on `centre`: at the centre from three
 * radii out or more; nearer, the view tips toward the horizon (toward the frame's `up` side), until just over the
 * ground the horizon sits a little above the middle of the picture. Returns a point on the new line of sight.
 */
export function grazeLook(eye: Vec3, centre: Vec3, r: number, fovY: number, up: Vec3 = [0, 0, 1]): Vec3 {
  const to: Vec3 = [centre[0] - eye[0], centre[1] - eye[1], centre[2] - eye[2]];
  const d = Math.hypot(to[0], to[1], to[2]);
  if (!(r > 0) || !(d > r)) return centre;
  const g = Math.min(1, Math.max(0, (3 - d / r) / 2));
  if (g === 0) return centre;
  const c = norm(to);
  // The horizon lies asin(r/d) from the centre's direction; aim a little short of it so the ground fills the lower part.
  const phi = g * Math.max(0, Math.asin(Math.min(1, r / d)) - 0.3 * fovY);
  let t: Vec3 = [up[0] - dot(up, c) * c[0], up[1] - dot(up, c) * c[1], up[2] - dot(up, c) * c[2]];
  if (Math.hypot(t[0], t[1], t[2]) < 1e-6) t = Math.abs(c[0]) < 0.9 ? [1 - c[0] * c[0], -c[0] * c[1], -c[0] * c[2]] : [-c[1] * c[0], 1 - c[1] * c[1], -c[1] * c[2]];
  t = norm(t);
  const k = Math.cos(phi);
  const s = Math.sin(phi);
  return [eye[0] + d * (c[0] * k + t[0] * s), eye[1] + d * (c[1] * k + t[1] * s), eye[2] + d * (c[2] * k + t[2] * s)];
}

/** The ground under the camera, for the detail ladder (VP3 part 4) to read finer tiles there first. */
export interface GroundView {
  /** The world's id. */
  body: string;
  /** Unit direction from the world's centre to the point under the camera, in the world's own frame. */
  toward: Vec3;
  /** The ground one screen pixel spans there, metres (straight down; a grazing view's far ground is coarser). */
  m_per_px: number;
  /** The camera's height over the ground, metres. */
  height_m: number;
}

/**
 * The ground under a camera at `eye` over a world of drawn radius `r` (and true radius `radius_m`) centred on
 * `centre`, seen with a vertical field `fovY` across `height_px` pixels; null when the camera is not near it (beyond
 * `CLOSE_REACH` radii) or the world's size is not known. The world's frame is the drawing's (no spin, as System3D).
 */
export function groundUnder(body: string, eye: Vec3, centre: Vec3, r: number, radius_m: number | null | undefined, fovY: number, height_px: number): GroundView | null {
  if (!(r > 0) || !(radius_m && radius_m > 0)) return null;
  const v: Vec3 = [eye[0] - centre[0], eye[1] - centre[1], eye[2] - centre[2]];
  const d = Math.hypot(v[0], v[1], v[2]);
  if (!(d > r) || d > CLOSE_REACH * r) return null;
  const height_m = ((d - r) / r) * radius_m;
  return { body, toward: norm(v), m_per_px: (height_m * 2 * Math.tan(fovY / 2)) / Math.max(1, height_px), height_m };
}

/** The orbit camera of a view about the world at its centre: turned `yaw` about the pole, raised `pitch`, at `dist`. */
export interface Orbit {
  yaw: number;
  pitch: number;
  dist: number;
}

/**
 * The orbit camera as a postcard's camera keeps it (PC2): the eye as a unit direction from the world's centre and a
 * distance in the world's radii (`r` its drawn radius). The view's up is the frame's pole, and where it looks follows
 * from these (`grazeLook`), so nothing more is needed to draw the same picture again.
 */
export function orbitEye(o: Orbit, r: number): { dir: Vec3; radii: number } | null {
  if (!(r > 0) || !(o.dist > 0)) return null;
  const c = Math.cos(o.pitch);
  return { dir: [c * Math.cos(o.yaw), c * Math.sin(o.yaw), Math.sin(o.pitch)], radii: o.dist / r };
}

/** The orbit camera that puts the eye at `dir` (from the centre), `radii` world radii out (the inverse of orbitEye). */
export function orbitOf(dir: Vec3, radii: number, r: number): Orbit {
  const d = norm(dir);
  return { yaw: Math.atan2(d[1], d[0]), pitch: Math.asin(Math.max(-1, Math.min(1, d[2]))), dist: radii * r };
}
