// What the 3D view draws, from either scene: bodies with a parent and a
// distance from it (metres, f64), never a flattened system-wide position.
// The view shows one frame at a time (rendering rule 2): a centre body and
// the bodies that orbit it, at log-scaled display distances.
import { KM_PER_AU, fact, parentOf, primaryOf, subjectName, type Mark, type PlayerScene, type PlayerSubject, type SceneSubject, type SystemScene } from "../../../ui-contract/src/index.ts";
import type { FrameNode } from "../../../renderer/src/index.ts";

export interface Body3D {
  id: string;
  parent: string | null;
  name: string;
  /** Distance from the parent, metres; null when the audience does not have it. */
  a_m: number | null;
  /** Phase angle, degrees; `phase_known` false when it is spread for display (players are not given phases). */
  phase_deg: number;
  phase_known: boolean;
  mark: Mark | null;
}

export function fromSystemScene(scene: SystemScene, mark: (s: SceneSubject) => Mark): Body3D[] {
  const star = primaryOf(scene);
  return scene.subjects
    .filter((s) => s === star || s.orbit)
    .map((s) => ({ id: s.address, parent: s === star ? null : (s.parent ?? star?.address ?? null), name: s.label, a_m: s.orbit ? s.orbit.a_au * KM_PER_AU * 1000 : null, phase_deg: s.orbit?.phase_deg ?? 0, phase_known: true, mark: mark(s) }));
}

export function fromPlayerScene(scene: PlayerScene, field: string, mark?: (s: PlayerSubject) => Mark): Body3D[] {
  const out: Body3D[] = [];
  const kids = new Map<string, number>();
  for (const s of scene.subjects) {
    const parent = s.address === scene.scope ? null : (parentOf(s.address, scene.scope) ?? scene.scope);
    const f = fact(s, field);
    const a = typeof f?.value === "number" ? f.value : f?.band ? (f.band[0] + f.band[1]) / 2 : null;
    const i = parent ? (kids.get(parent) ?? 0) : 0;
    if (parent) kids.set(parent, i + 1);
    out.push({ id: s.address, parent, name: subjectName(s), a_m: a, phase_deg: 25 + i * 67, phase_known: false, mark: mark ? mark(s) : null });
  }
  return out;
}

export const AU = KM_PER_AU * 1000;

export interface FrameLayout {
  centre: Body3D;
  placed: Body3D[];
  unplaced: Body3D[];
  nodes: FrameNode[];
  /** Display radius of each placed body's orbit (display units), by id. */
  ring: Map<string, number>;
  inner_m: number;
  outer_m: number;
}

/** Display units: orbits between 1.5 and 10, log-spaced. */
export function layout(bodies: readonly Body3D[], centreId: string): FrameLayout | null {
  const centre = bodies.find((b) => b.id === centreId);
  if (!centre) return null;
  const kids = bodies.filter((b) => b.parent === centreId);
  const placed = kids.filter((b) => b.a_m !== null && b.a_m > 0).sort((a, b) => a.a_m! - b.a_m!);
  const unplaced = kids.filter((b) => !placed.includes(b));
  const inner = placed.length ? placed[0]!.a_m! / 1.6 : 1;
  const outer = placed.length ? placed[placed.length - 1]!.a_m! * 1.25 : 10;
  const r = (a: number) => 1.5 + (8.5 * Math.log(a / inner)) / Math.log(outer / inner);
  const ring = new Map<string, number>();
  const nodes: FrameNode[] = [{ id: centre.id, parent: null, local: [0, 0, 0] }];
  for (const b of placed) {
    const d = r(b.a_m!);
    const t = (b.phase_deg * Math.PI) / 180;
    ring.set(b.id, d);
    nodes.push({ id: b.id, parent: centre.id, local: [d * Math.cos(t), d * Math.sin(t), 0] });
  }
  return { centre, placed, unplaced, nodes, ring, inner_m: inner, outer_m: outer };
}

export const hasChildren = (bodies: readonly Body3D[], id: string) => bodies.some((b) => b.parent === id && b.a_m !== null);
