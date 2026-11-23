// Measuring on the chart (V14): the distance between two bodies at the
// ruler's time, from the scene's own orbits (semi-major axis and phase,
// circular and in one plane, as the chart draws them), with the closest and
// farthest they can be when both circle the same parent, and light time.
// Presentation arithmetic on the scene's numbers only; no physics.
import { KM_PER_AU, type SceneSubject, type SystemScene } from "./scene.ts";
import { constant } from "./sheet.ts";

export const C_KM_S = constant("c") / 1000;

/** A body's position in AU relative to the system's root, summing its parents' orbits. */
function position(scene: SystemScene, s: SceneSubject): [number, number] {
  let x = 0;
  let y = 0;
  let cur: SceneSubject | undefined = s;
  for (let guard = 0; cur && guard < 8; guard++) {
    if (cur.orbit) {
      const ph = (cur.orbit.phase_deg * Math.PI) / 180;
      x += cur.orbit.a_au * Math.cos(ph);
      y += cur.orbit.a_au * Math.sin(ph);
    }
    const p: string | undefined = cur.parent;
    cur = p ? scene.subjects.find((q) => q.address === p) : undefined;
  }
  return [x, y];
}

export interface Separation {
  from: string;
  to: string;
  now_au: number;
  /** Closest and farthest over their orbits, when both circle one parent or one circles the other. */
  range_au?: [number, number];
  light_s: number;
}

export function separation(scene: SystemScene, a: string, b: string): Separation | null {
  const A = scene.subjects.find((s) => s.address === a);
  const B = scene.subjects.find((s) => s.address === b);
  if (!A || !B || A === B) return null;
  if (A.band_au || B.band_au) return null;
  const [ax, ay] = position(scene, A);
  const [bx, by] = position(scene, B);
  const now = Math.hypot(ax - bx, ay - by);
  let range: [number, number] | undefined;
  if (A.orbit && B.orbit && A.parent === B.parent) range = [Math.abs(A.orbit.a_au - B.orbit.a_au), A.orbit.a_au + B.orbit.a_au];
  else if (B.orbit && B.parent === A.address) range = [B.orbit.a_au, B.orbit.a_au];
  else if (A.orbit && A.parent === B.address) range = [A.orbit.a_au, A.orbit.a_au];
  else if (!A.orbit && !A.parent && B.orbit && !B.parent) range = [B.orbit.a_au, B.orbit.a_au];
  else if (!B.orbit && !B.parent && A.orbit && !A.parent) range = [A.orbit.a_au, A.orbit.a_au];
  return { from: A.label, to: B.label, now_au: now, range_au: range, light_s: (now * KM_PER_AU) / C_KM_S };
}

/** "1.23 AU (184 million km)" or "384,000 km" below a hundredth of an AU. */
export function distanceWords(au: number): string {
  const km = au * KM_PER_AU;
  const kmw = km >= 1e6 ? `${(km / 1e6).toLocaleString("en-GB", { maximumFractionDigits: km >= 1e8 ? 0 : 1 })} million km` : `${Math.round(km).toLocaleString("en-GB")} km`;
  if (au < 0.01) return kmw;
  return `${au.toLocaleString("en-GB", { maximumSignificantDigits: 3 })} AU (${kmw})`;
}

/** "8.3 light minutes", "1.3 light seconds", "4.2 light hours". */
export function lightWords(s: number): string {
  const f = (x: number) => x.toLocaleString("en-GB", { maximumSignificantDigits: 2 });
  if (s < 60) return `${f(s)} light seconds`;
  if (s < 3600) return `${f(s / 60)} light minutes`;
  if (s < constant("day") * 2) return `${f(s / 3600)} light hours`;
  return `${f(s / constant("day"))} light days`;
}
