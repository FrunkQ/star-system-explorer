// The builder's parts as shapes (CM14c, to the mock-up Alex accepted: sse4/handover-screens/CM-builder-mockup/): each
// placed part drawn as what it is, from its kind and its shape hint, at its own length and radius. A tank has domed
// ends, a habitat is a ribbed can with windows, an engine is a bell, a truss is a lattice, a panel is a flat wing
// reaching out from where it clips on, a dish is a bowl on a stalk. Geometry only: the sizes are the part's (its
// palette entry stretched by the recipe), the looks are the style's skin, and nothing here works out a figure.
import { BoxGeometry, type BufferGeometry, CylinderGeometry, LatheGeometry, SphereGeometry, TorusGeometry, Vector2 } from "three";
import { mergeGeometries } from "three/addons/utils/BufferGeometryUtils.js";
import type { PlacedPart } from "./ship-build.ts";
import type { Surface } from "./ship-mesh.ts";

/** Which look a piece of a part takes: a surface of the style's skin, or one of the few the skin has no entry for yet. */
export type Finish = Surface | "cells" | "window" | "dark";

export interface ShapePiece {
  geometry: BufferGeometry;
  finish: Finish;
}

const V2 = (r: number, x: number) => new Vector2(Math.max(r, 1e-4), x);
/** A lathe about the x axis: points are (radius, x), x from -L/2 (stern) to +L/2 (nose). */
const lathe = (pts: Vector2[], seg: number) => new LatheGeometry(pts, seg).rotateZ(-Math.PI / 2);
/** A frustum along x, centred, radius r0 at the stern and r1 at the nose. */
const cylX = (r0: number, r1: number, L: number, seg: number, open = false) => new CylinderGeometry(r1, r0, L, seg, 1, open).rotateZ(-Math.PI / 2);
const merge = (gs: BufferGeometry[]) => {
  const g = mergeGeometries(gs, false)!;
  for (const x of gs) x.dispose();
  return g;
};

/** The part's own shape, centred on its own axis along x, from -L/2 to +L/2 (panels and dishes reach along +y). */
function local(q: PlacedPart, seg: number, cut: boolean): ShapePiece[] {
  const s = q.part.shape;
  const L = s.length_m;
  const R = s.radius_m;
  const out: ShapePiece[] = [];
  const add = (geometry: BufferGeometry, finish: Finish) => out.push({ geometry, finish });
  const domed = (dome: number, finish: Finish) => {
    const d = Math.min(R * 0.9, L * 0.45, dome);
    const pts: Vector2[] = [];
    for (let i = 0; i <= 10; i++) {
      const a = (i / 10) * (Math.PI / 2);
      pts.push(V2(R * Math.sin(a), -L / 2 + d * (1 - Math.cos(a))));
    }
    for (let i = 10; i >= 0; i--) {
      const a = (i / 10) * (Math.PI / 2);
      pts.push(V2(R * Math.sin(a), L / 2 - d * (1 - Math.cos(a))));
    }
    add(lathe(pts, seg), finish);
  };
  const bands = (xs: number[], finish: Finish) => {
    const gs = xs.map((x) => new TorusGeometry(R * 1.005, Math.max(0.03, R * 0.025), 6, seg).rotateY(Math.PI / 2).translate(x, 0, 0));
    if (gs.length) add(merge(gs), finish);
  };
  if (s.look === "capsule") {
    // A crew capsule: a blunt cone from its heat-shield end, a dark band of tiles at the base, two windows.
    const prof = s.profile && s.profile.length > 1 ? s.profile : ([[0, 1], [0.1, 1], [0.9, 0.36], [1, 0.3]] as [number, number][]);
    add(lathe([V2(0, -L / 2), ...prof.map(([a, r]) => V2(r * R, -L / 2 + a * L)), V2(0, L / 2)], seg), "hull");
    add(lathe([V2(0, -L / 2 - R * 0.04), V2(R * 1.01, -L / 2 - R * 0.01), V2(R * 1.01, -L / 2 + L * 0.03), V2(0, -L / 2 + L * 0.03)], seg), "dark");
    add(merge([1, -1].map((side) => new BoxGeometry(R * 0.16, R * 0.11, R * 0.04).translate(-0.05 * L, side * R * 0.7, R * 0.2))), "window");
    return out;
  }
  if (s.look === "port") {
    add(cylX(R, R, L, seg), "drive");
    add(new TorusGeometry(R * 0.95, R * 0.12, 8, seg).rotateY(Math.PI / 2).translate(L / 2, 0, 0), "hull");
    return out;
  }
  switch (q.part.kind) {
    case "tank":
      domed(R * 0.7, "tanks");
      break;
    case "stage": {
      domed(R * 0.5, "tanks");
      const n = Math.max(1, Math.min(6, Math.round(L / (4 * R))));
      bands(Array.from({ length: n + 1 }, (_, i) => -L / 2 + R * 0.5 + (i * (L - R)) / n), "spine");
      break;
    }
    case "habitat": {
      if (cut) {
        // Cut away on the viewer's side (-y): the shell open from a little above the middle line to below it, its
        // ends and a deck inside showing.
        const open = 2.1;
        add(new CylinderGeometry(R, R, L, seg, 1, true, Math.PI / 2 + open / 2, Math.PI * 2 - open).rotateZ(-Math.PI / 2), "hull");
        add(merge([-L / 2, L / 2].map((x) => cylX(R, R, Math.max(0.02, L * 0.01), seg).translate(x, 0, 0))), "hull");
        add(new BoxGeometry(L * 0.96, R * 1.7, Math.max(0.04, R * 0.04)).translate(0, 0, -R * 0.45), "spine");
        break;
      }
      // A can: rounded shoulders, a raised rib every few metres, two rows of windows.
      const pts = [V2(0, -L / 2), V2(R * 0.86, -L / 2), V2(R, -L / 2 + Math.min(0.3, L * 0.08))];
      const rib = Math.max(1.6, R * 1.3);
      for (let x = -L / 2 + rib; x < L / 2 - rib * 0.5; x += rib) pts.push(V2(R, x - R * 0.06), V2(R * 1.025, x - R * 0.03), V2(R * 1.025, x + R * 0.03), V2(R, x + R * 0.06));
      pts.push(V2(R, L / 2 - Math.min(0.3, L * 0.08)), V2(R * 0.86, L / 2), V2(0, L / 2));
      add(lathe(pts, seg), "hull");
      const w: BufferGeometry[] = [];
      const step = Math.max(0.9, R * 0.9);
      for (let x = -L / 2 + step; x < L / 2 - step * 0.6; x += step)
        for (const side of [1, -1]) w.push(new BoxGeometry(R * 0.28, R * 0.16, R * 0.06).translate(x, side * R * 0.97, R * 0.25));
      if (w.length) add(merge(w), "window");
      break;
    }
    case "pod": {
      const h = (s.span_m ?? 2 * R) / 2;
      add(new BoxGeometry(L, 2 * h, 2 * h), "hull");
      const gs: BufferGeometry[] = [];
      for (let x = -L / 2 + L * 0.12; x < L / 2; x += Math.max(0.6, L / 4)) gs.push(new BoxGeometry(Math.max(0.05, L * 0.03), 2 * h * 1.04, 2 * h * 1.04).translate(x, 0, 0));
      add(merge(gs), "spine");
      break;
    }
    case "node": {
      add(new SphereGeometry(R * 0.92, seg, Math.max(8, seg / 2)), "hull");
      const ports = [0, 90, 180, 270].map((c) => cylX(R * 0.42, R * 0.42, R * 0.5, seg).rotateZ(Math.PI / 2).translate(0, R * 0.95, 0).rotateX((c * Math.PI) / 180));
      add(merge([...ports, cylX(R * 0.45, R * 0.45, L, seg)]), "drive");
      break;
    }
    case "truss": {
      const a = Math.max(R * 0.7, 0.05);
      const bar = Math.max(0.02, a * 0.16);
      const gs: BufferGeometry[] = [];
      for (const [y, z] of [[a, a], [a, -a], [-a, a], [-a, -a]] as const) gs.push(new BoxGeometry(L, bar, bar).translate(0, y, z));
      const n = Math.max(1, Math.min(60, Math.round(L / (2 * a))));
      for (let i = 0; i <= n; i++) {
        const x = -L / 2 + (i * L) / n;
        gs.push(new BoxGeometry(bar * 0.8, 2 * a, bar * 0.8).translate(x, 0, a), new BoxGeometry(bar * 0.8, 2 * a, bar * 0.8).translate(x, 0, -a), new BoxGeometry(bar * 0.8, bar * 0.8, 2 * a).translate(x, a, 0), new BoxGeometry(bar * 0.8, bar * 0.8, 2 * a).translate(x, -a, 0));
      }
      for (let i = 0; i < n; i++) {
        const x = -L / 2 + ((i + 0.5) * L) / n;
        const len = Math.hypot(L / n, 2 * a);
        for (const z of [a, -a]) gs.push(new BoxGeometry(len, bar * 0.5, bar * 0.5).rotateZ(Math.atan2(2 * a, L / n) * (i % 2 ? 1 : -1)).translate(x, 0, z));
      }
      add(merge(gs), "spine");
      break;
    }
    case "solar":
    case "radiator": {
      // A wing reaching out (+y) from its root, its width along the ship; a short boom where it meets the carrier.
      const span = s.span_m ?? Math.max(4 * R, L);
      const t = Math.max(0.04, Math.min(0.12, L * 0.02));
      const boom = Math.min(0.3, span * 0.03);
      add(new BoxGeometry(L, span - boom, t).translate(0, (span + boom) / 2, 0), q.part.kind === "solar" ? "cells" : "radiators");
      add(cylX(t, t, boom * 1.2, 8).rotateZ(Math.PI / 2).translate(0, boom / 2, 0), "spine");
      break;
    }
    case "dish": {
      // A bowl facing out (+y) on a stalk.
      const pts: Vector2[] = [];
      const depth = Math.max(L * 0.6, R * 0.25);
      for (let i = 0; i <= 10; i++) {
        const r = (R * i) / 10;
        pts.push(V2(r, depth * 0.3 + ((r * r) / (R * R)) * depth * 0.7));
      }
      add(new LatheGeometry(pts, seg), "hull");
      add(new CylinderGeometry(Math.max(0.03, R * 0.04), Math.max(0.03, R * 0.04), depth * 0.6, 8).translate(0, depth * 0.3, 0), "spine");
      break;
    }
    case "engine": {
      // A bell: wide at the stern, a throat, a chamber and its mount ahead (the profile's radii, as shares of R).
      const prof = s.profile && s.profile.length > 1 ? s.profile : ([[0, 1], [0.7, 0.45], [1, 0.35]] as [number, number][]);
      const [ta, tr] = prof.length > 2 ? prof[prof.length - 2]! : [0.7, 0.4];
      const pts: Vector2[] = [];
      for (let i = 0; i <= 12; i++) {
        const t = i / 12;
        pts.push(V2(R * (tr * 0.75 + (1 - tr * 0.75) * (1 - t) ** 1.6), -L / 2 + t * ta * L));
      }
      const top = prof[prof.length - 1]![1] * R;
      pts.push(V2(top, -L / 2 + ta * L + L * 0.05), V2(top, L / 2), V2(0, L / 2));
      add(lathe(pts, seg), "drive");
      break;
    }
    case "power": {
      add(cylX(R, R, L * 0.75, seg).translate(-L * 0.08, 0, 0), "hull");
      const fins: BufferGeometry[] = [];
      for (let i = 0; i < 6; i++) fins.push(new BoxGeometry(L * 0.6, Math.max(0.02, R * 0.06), R * 0.55).translate(-L * 0.08, 0, R * 1.2).rotateX((i * Math.PI) / 3));
      add(merge(fins), "radiators");
      add(cylX(R * 0.5, R * 0.5, L * 0.25, seg).translate(L * 0.375, 0, 0), "spine");
      break;
    }
    case "bay": {
      // An open bay: a shell with a gap along its top, a ring at each end, a back wall.
      const gap = (80 * Math.PI) / 180;
      // Turned onto x, angle 0 is straight up (+z): the gap is centred there.
      // Cut away, it is open on the viewer's side (-y, angle 90 degrees) as well, so what it holds shows.
      const from = cut ? Math.PI / 2 + 0.7 : gap / 2;
      add(new CylinderGeometry(R, R, L, seg, 1, true, from, Math.PI * 2 - gap / 2 - from).rotateZ(-Math.PI / 2), "hull");
      add(merge([-L / 2, L / 2].map((x) => new TorusGeometry(R, Math.max(0.04, R * 0.06), 6, seg).rotateY(Math.PI / 2).translate(x, 0, 0))), "spine");
      add(cylX(R * 0.98, R * 0.98, Math.max(0.02, L * 0.01), seg).translate(-L / 2, 0, 0), "dark");
      break;
    }
    case "sensor": {
      add(new BoxGeometry(L, 1.6 * R, 1.6 * R), "hull");
      add(merge([[-0.25, 0.35], [0.1, -0.35], [0.3, 0.3]].map(([x, z]) => new SphereGeometry(R * 0.3, 12, 8, 0, Math.PI * 2, 0, Math.PI / 2).translate(x! * L, 0.8 * R, z! * R))), "hull");
      add(cylX(R * 0.25, R * 0.25, R * 0.3, 12).translate(L / 2 + R * 0.15, 0, 0), "dark");
      break;
    }
    case "shield":
      if (s.profile && s.profile.length > 1) {
        // A shadow shield: a cone, wide towards the crew.
        add(lathe([V2(0, -L / 2), ...s.profile.map(([a, r]) => V2(r * R, -L / 2 + a * L)), V2(0, L / 2)], seg), "spine");
        break;
      }
      add(lathe([V2(0, -L / 2 - R * 0.05), V2(R * 0.7, -L / 2), V2(R, -L / 2 + 0.3 * L), V2(R, L / 2), V2(0, L / 2)], seg), "shield");
      break;
    case "legs":
      // A strut with its foot pad at the stern end (a lander stands on its tail).
      add(merge([cylX(R, R, L, 10), cylX(R * 4.5, R * 4.5, Math.max(0.05, L * 0.05), 16).translate(-L / 2, 0, 0)]), "spine");
      break;
    default: {
      // Any other lathe follows its own profile; without one, a can.
      const prof = s.profile && s.profile.length > 1 ? s.profile : ([[0, 1], [1, 1]] as [number, number][]);
      add(lathe([V2(0, -L / 2), ...prof.map(([a, r]) => V2(r * R, -L / 2 + a * L)), V2(0, L / 2)], seg), "hull");
    }
  }
  return out;
}

/** A placed part's shapes in the build's frame (x along the ship from the stern, z up): the part's own shape moved to
 * where it sits, flat parts turned to reach out along their clock, hull parts facing out. `cut` opens bays and habitats on
 * the viewer's side (the mock-up's Cutaway), so what they hold shows. */
export function partShapes(q: PlacedPart, seg: number, o: { cut?: boolean } = {}): ShapePiece[] {
  const L = q.part.shape.length_m;
  const flat = q.part.kind === "solar" || q.part.kind === "radiator" || q.part.kind === "dish";
  const pieces = local(q, seg, o.cut === true);
  for (const p of pieces) {
    if (flat) {
      // Reach out along the clock: +y turned onto (sin c, cos c) in (y, z). On the axis a dish faces forward.
      if (q.part.kind === "dish" && q.built.place !== "hull") p.geometry.rotateZ(-Math.PI / 2);
      else p.geometry.rotateX(((90 - q.roll_deg) * Math.PI) / 180);
    }
    p.geometry.translate(q.x0 + L / 2, q.y, q.z);
  }
  return pieces;
}

/** The part a ship's name is lettered on: its biggest crew or tank part, one on the axis or the viewer's side first. */
export function nameCarrier(placed: readonly PlacedPart[]): PlacedPart | null {
  let best: PlacedPart | null = null;
  let score = 0;
  for (const q of placed) {
    if (!["habitat", "tank", "stage", "bay"].includes(q.part.kind)) continue;
    const near = q.y <= 1e-6 ? 1 : 0.4;
    const a = q.part.shape.length_m * q.part.shape.radius_m * (q.part.kind === "habitat" ? 3 : 1) * near;
    if (a > score) (score = a), (best = q);
  }
  return best;
}
