// The journey played (GZ3 and GZ8, first part: design/SSE4-grand-zoom-design.md sections 5.3 to 5.7). journey.ts lays a
// journey out and says, for any journey time, each view's camera; this file is what the shell's player and the
// views' journey ports need besides: how far the player may go while a view is still reading (5.4), the place line
// as the progress bar (5.5), the one line of width while moving (5.7), and each travelled view's own answer to "put
// your camera here" (the web cube's eye, the galaxies nearby's turn and reach). Pure: no clock, no DOM. Presentation
// only: never read by physics, never in the state hash.
import { say, sayNumber } from "./figures.ts";
import { constant } from "./sheet.ts";
import { RUNGS, RUNG_INFO, RUNG_WORDS, rungRank, type LadderRung } from "./approach.ts";
import { type JourneyCamera, type JourneyFrame, type JourneyPlan } from "./journey.ts";
import type { NearbyTurn } from "./galaxies-nearby.ts";
import type { Vec3 } from "./pose.ts";

const MPC = 1e6 * constant("pc");

/** A hair before a seam: where the player holds while the incoming view is still reading (5.4: never half-faded). */
export const SEAM_HOLD_S = 1e-3;

/**
 * How far the player may go while views are still reading (design 5.4): never into the fade of a rung whose view is
 * not ready. The first rung not ready holds the journey just before its seam; when it is the first rung of all, the
 * journey holds at its start. `ready[i]` is the port's answer for `plan.path[i]`.
 */
export function readyHold(plan: JourneyPlan, t: number, ready: readonly boolean[]): { t: number; waiting: number | null } {
  for (let i = 0; i < plan.path.length; i++) {
    if (ready[i]) continue;
    const e = plan.path[i]!;
    const cap = i === 0 ? 0 : Math.max(0, e.t_in - SEAM_HOLD_S);
    if (t >= cap) return { t: cap, waiting: i };
    return { t, waiting: null };
  }
  return { t, waiting: null };
}

/** One step of the place line drawn as the journey's progress bar (5.5): passed in ink, in view in the accent, to
 * come pencilled; a rung no view draws yet is pencilled and says why. */
export interface JourneyStep {
  rung: LadderRung;
  words: string;
  state: "passed" | "current" | "coming" | "held" | "off";
  /** The entry of the plan this step is (absent for a rung the journey does not pass). */
  entry?: number;
  /** Why a held rung is passed over. */
  why?: string;
}

/** The seven rungs as the progress bar, for one moment of the journey. Rungs above the start and below the end are
 * "off" (drawn faint, still pressable to start a new journey there in a later package). */
export function journeySteps(plan: JourneyPlan, frame: Pick<JourneyFrame, "entry">): JourneyStep[] {
  const first = rungRank(plan.path[0]!.rung);
  const last = rungRank(plan.path.at(-1)!.rung);
  return RUNGS.map((rung) => {
    const words = RUNG_WORDS[rung];
    const entry = plan.path.findIndex((p) => p.rung === rung);
    if (entry >= 0) return { rung, words, entry, state: entry < frame.entry ? "passed" : entry === frame.entry ? "current" : "coming" };
    const r = rungRank(rung);
    if (r > first && r < last) return { rung, words, state: "held", why: RUNG_INFO[rung].held ?? `${words} is not drawn here yet` };
    return { rung, words, state: "off" };
  });
}

/** Which reading of a length suits a picture this wide: light years above about a light year, then the system's and
 * the world's own units (the figures door picks the rung within each). */
function aboutFor(width_m: number): { class?: string; what?: string } {
  if (width_m >= 1e16) return { class: "galaxy" };
  if (width_m >= 1e10) return { what: "orbit" };
  return {};
}

/** The one line of HUD while moving (design 5.7): the picture's width in words and as a power of ten,
 * "about 10 million light years across · 3.1 × 10²³ m". */
export function widthLine(width_m: number): { words: string; power: string; exp: number } {
  let exp = Math.floor(Math.log10(width_m));
  let lead = Math.round((width_m / 10 ** exp) * 10) / 10;
  if (lead >= 10) ((lead = 1), (exp += 1));
  const words = `${say(width_m, "length", aboutFor(width_m), { depth: "curious", sig: 2 }).text} across`;
  return { words, power: `${sayNumber(lead, 2)} × 10${superscript(exp)} m`, exp };
}
const SUP: Record<string, string> = { "-": "⁻", "0": "⁰", "1": "¹", "2": "²", "3": "³", "4": "⁴", "5": "⁵", "6": "⁶", "7": "⁷", "8": "⁸", "9": "⁹" };
const superscript = (n: number) => String(n).replace(/./g, (c) => SUP[c] ?? c);

// --- the ports' answers ----------------------------------------------------------------------------------------

/** The web cube's field of view, top to bottom (Web3D draws with it; its journey port aims with it). */
export const WEB_CUBE_FOV = (40 * Math.PI) / 180;

/**
 * The web cube's camera for a journey (its port's `aim`): the eye on the line of sight from the target, at the
 * distance that shows the picture's width across the drawing's shorter side through the cube's field of view. In the
 * cube's own units (comoving Mpc), about the target given in them. The web's axes are the galactic frame's (F37), the
 * frame the journey's directions are given in for the web and the galaxies nearby alike.
 */
export function webEye(cam: Pick<JourneyCamera, "width_m" | "eye_dir">, target: Vec3, fovY: number): { eye: Vec3; dist_mpc: number } {
  const dist = cam.width_m / MPC / 2 / Math.tan(fovY / 2);
  const d = cam.eye_dir;
  return { eye: [target[0] + d[0] * dist, target[1] + d[1] * dist, target[2] + d[2] * dist], dist_mpc: dist };
}

/**
 * The galaxies nearby's camera for a journey: the reach that shows the picture's width across the drawing (it is
 * drawn flat, `size` across with `pad` kept clear), and the turn whose line of sight is the journey's. The inverse of
 * `projectNearby`'s turn: tipped `tilt` from straight above our disc, turned `yaw` about its axis; the page's up is
 * the disc's axis as seen, the same up the web cube's camera keeps (z up, no roll), so the two agree at a seam.
 */
export function nearbyAim(cam: Pick<JourneyCamera, "width_m" | "eye_dir">, size: number, pad = 24): { reach_mpc: number; turn: NearbyTurn } {
  const reach = ((cam.width_m / MPC) * (size / 2 - pad)) / size;
  const [x, y, z] = cam.eye_dir;
  const tilt = (Math.acos(Math.max(-1, Math.min(1, z))) * 180) / Math.PI;
  const flat = Math.hypot(x, y);
  const yaw = flat < 1e-9 ? 0 : (Math.atan2(-x, -y) * 180) / Math.PI;
  return { reach_mpc: reach, turn: { yaw_deg: yaw, tilt_deg: Math.max(0, Math.min(90, tilt)) } };
}

/**
 * Our galaxy in the web cube (the web port's target). The cube's galaxies are the catalogue's haloes, each moved by
 * the cube's own Zel'dovich step from where it collapsed; the home galaxy is a catalogued halo at its place before
 * that step. So home in the cube is the cube galaxy with home's mass now that sits nearest home's own place: the same
 * halo, found through the step the cube applied (GZ0 measured the step at up to about 7 Mpc; GZ15 will give every
 * halo one place, and this becomes a plain look-up). Null when the cube holds no galaxy of home's mass (home below
 * the cube's light floor, or outside it).
 */
export function homeInCube(
  galaxies: readonly { position: readonly [number, number, number]; halo_msun: number }[],
  home: { position_mpc: readonly [number, number, number]; mass_msun: number },
  tolerance = 1e-6,
): { position: Vec3; moved_mpc: number } | null {
  let best: { position: Vec3; moved_mpc: number } | null = null;
  for (const g of galaxies) {
    if (!(Math.abs(g.halo_msun / home.mass_msun - 1) <= tolerance)) continue;
    const moved = Math.hypot(g.position[0] - home.position_mpc[0], g.position[1] - home.position_mpc[1], g.position[2] - home.position_mpc[2]);
    if (!best || moved < best.moved_mpc) best = { position: [g.position[0], g.position[1], g.position[2]], moved_mpc: moved };
  }
  return best;
}
