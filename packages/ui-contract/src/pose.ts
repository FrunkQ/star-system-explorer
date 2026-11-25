// The camera on a bookmark (PC1, SSE4-postcards-design.md section 3). A bookmark already says where, when, which
// instrument and how it is looked at; the pose adds the exact camera, so a bookmark made in Free view comes back to
// the same picture and a postcard draws the same picture anywhere. Stored the way the renderer builds its camera
// (renderer thread, design section 14): the body the camera is about, the eye as a direction in that body's frame
// (z the pole) and a distance from its centre in metres, up, the field of view, the exposure and the part of the frame
// that must stay in view. Presentation only: never read by physics, never in the state hash.

export type Vec3 = [number, number, number];

/** How the picture is exposed. "long" is a long exposure (the picture's lift), "eye" what an eye would see; `ev` is
 * 0 until exposure is metered (VP4). */
export interface PoseExposure {
  mode: "long" | "eye";
  ev: number;
}

export interface Pose {
  /** The address of the body (or system, galaxy) whose frame the camera is in. */
  about: string;
  /** Unit vector from the body's centre towards the eye, in the body's frame (z the pole). */
  eye_dir: Vec3;
  /** Distance of the eye from the body's centre, metres. */
  eye_m: number;
  /** Unit vector that is up on the picture, in the body's frame. */
  up: Vec3;
  /** Field of view from the top of the kept box to its bottom, degrees. */
  fov_deg: number;
  exposure: PoseExposure;
  /** The part of the frame that must stay in view on any screen shape, as [left, top, right, bottom] fractions. */
  keep: [number, number, number, number];
}

export const POSE_FOV_MIN = 0.01;
export const POSE_FOV_MAX = 170;
/** The whole frame, the default keep box. */
export const KEEP_ALL: Pose["keep"] = [0, 0, 1, 1];

const finite = (x: unknown): x is number => typeof x === "number" && Number.isFinite(x);
const vec = (v: unknown): v is Vec3 => Array.isArray(v) && v.length === 3 && v.every(finite);
const len = (v: Vec3) => Math.hypot(v[0], v[1], v[2]);
const unit = (v: Vec3): Vec3 => {
  const l = len(v);
  return [v[0] / l, v[1] / l, v[2] / l];
};
const dot = (a: Vec3, b: Vec3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];

/** What is wrong with a pose, one line; null when it is a good one. */
export function checkPose(raw: unknown): string | null {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) return "the camera is not an object";
  const o = raw as Record<string, unknown>;
  if (typeof o.about !== "string" || !o.about) return "the camera does not say what it is about";
  if (!vec(o.eye_dir) || len(o.eye_dir) < 1e-9) return "the camera has no direction";
  if (!finite(o.eye_m) || o.eye_m <= 0) return "the camera has no distance";
  if (!vec(o.up) || len(o.up) < 1e-9) return "the camera has no up";
  const e = unit(o.eye_dir);
  const u = unit(o.up);
  if (Math.abs(dot(e, u)) > 0.999) return "the camera's up is along its line of sight";
  if (!finite(o.fov_deg) || o.fov_deg < POSE_FOV_MIN || o.fov_deg > POSE_FOV_MAX) return "the camera's field of view is out of range";
  const x = o.exposure as Record<string, unknown> | undefined;
  if (!x || typeof x !== "object" || (x.mode !== "long" && x.mode !== "eye") || !finite(x.ev)) return "the camera's exposure is not one";
  const k = o.keep;
  if (!Array.isArray(k) || k.length !== 4 || !k.every(finite)) return "the camera's kept box is not four numbers";
  const [l, t, r, b] = k as number[];
  if (l! < 0 || t! < 0 || r! > 1 || b! > 1 || r! <= l! || b! <= t!) return "the camera's kept box is outside the frame";
  return null;
}

/** A pose read from a save, a pack or a postcard: null when it is not a good one (the bookmark keeps going without
 * it). Directions come back as unit vectors, and up is made square to the line of sight. */
export function poseFrom(raw: unknown): Pose | null {
  if (checkPose(raw) !== null) return null;
  const o = raw as Pose;
  const eye = unit(o.eye_dir);
  const u0 = unit(o.up);
  const d = dot(eye, u0);
  const up = unit([u0[0] - d * eye[0], u0[1] - d * eye[1], u0[2] - d * eye[2]]);
  return {
    about: o.about,
    eye_dir: eye,
    eye_m: o.eye_m,
    up,
    fov_deg: o.fov_deg,
    exposure: { mode: o.exposure.mode, ev: o.exposure.ev },
    keep: [o.keep[0], o.keep[1], o.keep[2], o.keep[3]],
  };
}

const RAD = Math.PI / 180;

/** The globe's camera as a pose: the globe turned by `lon` about its pole and tipped by `lat` (the renderer's
 * viewRotation), seen from `eye_m` from the centre. The globe is drawn whole, so the field of view is what a body of
 * `radius_m` fills from there, with a little margin. */
export function poseOfGlobe(about: string, lon_deg: number, lat_deg: number, radius_m: number, opts: { eye_m?: number; exposure?: PoseExposure } = {}): Pose {
  const lo = lon_deg * RAD;
  const la = lat_deg * RAD;
  const eye_m = opts.eye_m ?? radius_m * 4;
  const fov = (2 * Math.asin(Math.min(1, radius_m / eye_m)) * 1.1) / RAD;
  return {
    about,
    eye_dir: [Math.cos(la) * Math.cos(lo), Math.cos(la) * Math.sin(lo), Math.sin(la)],
    eye_m,
    up: [-Math.sin(la) * Math.cos(lo), -Math.sin(la) * Math.sin(lo), Math.cos(la)],
    fov_deg: Math.min(POSE_FOV_MAX, Math.max(POSE_FOV_MIN, fov)),
    exposure: opts.exposure ?? { mode: "long", ev: 0 },
    keep: [...KEEP_ALL],
  };
}

/** The globe's turn and tip that look along a pose (the inverse of poseOfGlobe), in degrees, lon in [-180, 180). */
export function globeOfPose(p: Pose): { lon: number; lat: number } {
  const [x, y, z] = unit(p.eye_dir);
  const lat = Math.asin(Math.max(-1, Math.min(1, z))) / RAD;
  let lon = Math.atan2(y, x) / RAD;
  if (lon >= 180) lon -= 360;
  return { lon, lat };
}

/** The field of view that keeps the kept box in a screen of this shape (width over height): the camera's direction
 * stays, the view opens until the box fits (design section 3). */
export function fovToFit(p: Pose, aspect: number): number {
  const [l, t, r, b] = p.keep;
  const kw = r - l;
  const kh = b - t;
  if (!(aspect > 0) || kw <= 0 || kh <= 0) return p.fov_deg;
  // The pose's field of view spans the kept box's height; a screen narrower than the box opens it until the width fits.
  const keptAspect = kw / kh;
  const need = keptAspect > aspect ? keptAspect / aspect : 1;
  const half = Math.tan((p.fov_deg * RAD) / 2) * need;
  return Math.min(POSE_FOV_MAX, (2 * Math.atan(half)) / RAD);
}
