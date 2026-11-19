// Nested frames and the camera-relative rule (handover/rendering-rules.md).
// Every position is local to its parent frame, in f64 metres. Nothing here
// ever forms a "world" coordinate that adds a small offset to a huge one:
// the position of one node seen from another is summed along the tree path
// between them, from the common ancestor down each side, and the camera's
// own offset is subtracted in f64 before anything is narrowed to float32
// (the precision a GPU, or an SVG, works in).

export type Vec3 = [number, number, number];

export interface FrameNode {
  id: string;
  /** Null for the root (a patch, or the system when nothing is above it). */
  parent: string | null;
  /** Position of this node's origin in its parent's frame, metres (f64). */
  local: Vec3;
}

export type Tree = ReadonlyMap<string, FrameNode>;

export function tree(nodes: readonly FrameNode[]): Tree {
  return new Map(nodes.map((n) => [n.id, n]));
}

/** Root to `id`, inclusive. */
export function pathTo(t: Tree, id: string): FrameNode[] {
  const out: FrameNode[] = [];
  for (let n = t.get(id); n; n = n.parent === null ? undefined : t.get(n.parent)) out.push(n);
  return out.reverse();
}

const add = (a: Vec3, b: Vec3): Vec3 => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
const sub = (a: Vec3, b: Vec3): Vec3 => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];

/**
 * Position of `to`'s origin in the frame of `from`, in f64, summing only the
 * links below their common ancestor. Two moons of one planet never see the
 * star's distance from the patch centre in their arithmetic.
 */
export function relative(t: Tree, from: string, to: string): Vec3 {
  const a = pathTo(t, from);
  const b = pathTo(t, to);
  let i = 0;
  while (i < a.length && i < b.length && a[i]!.id === b[i]!.id) i++;
  let up: Vec3 = [0, 0, 0];
  for (let k = i; k < a.length; k++) up = add(up, a[k]!.local);
  let down: Vec3 = [0, 0, 0];
  for (let k = i; k < b.length; k++) down = add(down, b[k]!.local);
  return sub(down, up);
}

export interface Camera {
  /** The frame the camera sits in. */
  frame: string;
  /** Camera position in that frame, metres (f64). */
  eye: Vec3;
  /** Point looked at, in that frame, metres (f64). */
  look: Vec3;
  /** Vertical field of view, radians. */
  fovY: number;
  width: number;
  height: number;
}

/** Screen position in pixels (origin top left), or null behind the camera. */
export type Pixel = { x: number; y: number; depth: number } | null;

/** The camera's forward, right and up unit vectors (z is the orbital pole). */
export function basis(eye: Vec3, look: Vec3): { f: Vec3; r: Vec3; u: Vec3 } {
  const norm = (v: Vec3): Vec3 => {
    const l = Math.hypot(v[0], v[1], v[2]);
    return [v[0] / l, v[1] / l, v[2] / l];
  };
  const f = norm(sub(look, eye));
  // World up is +z (the system's orbital pole); fall back to +y when looking straight down it.
  const up: Vec3 = Math.abs(f[2]) > 0.999 ? [0, 1, 0] : [0, 0, 1];
  const r = norm([f[1] * up[2] - f[2] * up[1], f[2] * up[0] - f[0] * up[2], f[0] * up[1] - f[1] * up[0]]);
  const u: Vec3 = [r[1] * f[2] - r[2] * f[1], r[2] * f[0] - r[0] * f[2], r[0] * f[1] - r[1] * f[0]];
  return { f, r, u };
}

/** Perspective projection of a camera-relative point (metres), with every operation in `num` precision. */
function projectRel(rel: Vec3, cam: Camera, num: (x: number) => number): Pixel {
  const { f, r, u } = basis(cam.eye, cam.look);
  const dot = (a: Vec3, b: Vec3) => num(num(num(a[0] * b[0]) + num(a[1] * b[1])) + num(a[2] * b[2]));
  const B = (v: Vec3): Vec3 => [num(v[0]), num(v[1]), num(v[2])];
  const z = dot(rel, B(f));
  if (!(z > 0)) return null;
  const k = num(cam.height / 2 / Math.tan(cam.fovY / 2));
  return { x: num(cam.width / 2 + num(num(dot(rel, B(r)) * k) / z)), y: num(cam.height / 2 - num(num(dot(rel, B(u)) * k) / z)), depth: z };
}

/**
 * The camera-relative rule: the body's position relative to the camera is
 * found in f64 (`relative` plus the eye offset), then narrowed to float32 and
 * projected in float32, as the GPU would.
 */
export function project(t: Tree, cam: Camera, id: string, at: Vec3 = [0, 0, 0]): Pixel {
  const rel = sub(add(relative(t, cam.frame, id), at), cam.eye);
  return projectRel([Math.fround(rel[0]), Math.fround(rel[1]), Math.fround(rel[2])], cam, Math.fround);
}

/** The reference answer: everything in f64. */
export function projectExact(t: Tree, cam: Camera, id: string, at: Vec3 = [0, 0, 0]): Pixel {
  const rel = sub(add(relative(t, cam.frame, id), at), cam.eye);
  return projectRel(rel, cam, (x) => x);
}

/**
 * What the rule forbids, kept so the test can show it fails: world positions
 * from the root narrowed to float32, and the camera subtracted on the GPU.
 */
export function projectFlattened(t: Tree, cam: Camera, id: string, at: Vec3 = [0, 0, 0]): Pixel {
  const root = pathTo(t, cam.frame)[0]!.id;
  const w = add(relative(t, root, id), at);
  const e = add(relative(t, root, cam.frame), cam.eye);
  const f32 = Math.fround;
  const rel: Vec3 = [f32(f32(w[0]) - f32(e[0])), f32(f32(w[1]) - f32(e[1])), f32(f32(w[2]) - f32(e[2]))];
  return projectRel(rel, cam, f32);
}

/** The same camera re-expressed in another frame (rule 4: zoom changes the frame). */
export function rebase(t: Tree, cam: Camera, frame: string): Camera {
  if (frame === cam.frame) return cam;
  const o = relative(t, frame, cam.frame);
  return { ...cam, frame, eye: add(o, cam.eye), look: add(o, cam.look) };
}

/**
 * The deepest frame on the path to `target` whose origin is close enough
 * for float32 to carry the scene around the camera: a node is entered when
 * the camera is within `reach` times its distance from its own parent.
 */
export function frameFor(t: Tree, cam: Camera, target: string, reach = 4): string {
  const path = pathTo(t, target);
  let best = path[0]!.id;
  for (const n of path.slice(1)) {
    const d = relative(t, n.id, cam.frame);
    const e = add(d, cam.eye);
    const span = Math.hypot(...n.local) || 1;
    if (Math.hypot(...e) <= reach * span) best = n.id;
  }
  return best;
}
