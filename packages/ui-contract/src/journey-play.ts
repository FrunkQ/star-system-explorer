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
import { readyLine, warmPlan, type BuildTimes, type WarmHost, type WarmItem } from "./settle.ts";
import type { Vec3 } from "./pose.ts";

const MPC = 1e6 * constant("pc");

/** A hair before a seam: where the player holds while the incoming view is still reading (5.4: never half-faded). */
export const SEAM_HOLD_S = 1e-3;

/**
 * A level's answer to "are you ready" (visual flourish FL5, design SSE4-visual-flourish-design.md section 6): its
 * proper picture, a coarse one, or not yet. A coarse picture says how many things it shows and how many the proper
 * picture shows of the same (null when that is not known yet): by Alex's filled-in rule of 27 Nov it must show the
 * same things in the same places, so one with fewer, or one that cannot say, is not travelled on.
 */
export type LevelAnswer = { state: "proper" } | { state: "coarse"; things: number; of: number | null } | { state: "none" };

/** How the journey is being played: gone early (Go pressed before every level was ready), or for film. */
export interface PlayHow {
  early: boolean;
  film: boolean;
}
const WAITS: PlayHow = { early: false, film: false };

const answerOf = (a: boolean | LevelAnswer | undefined): LevelAnswer => (a === true ? { state: "proper" } : a === false || a === undefined ? { state: "none" } : a);

/** Whether a coarse picture keeps the filled-in rule: the same things as the proper one, none fewer. */
export function coarseFits(a: LevelAnswer): boolean {
  return a.state === "coarse" && a.of !== null && a.things >= a.of && a.things > 0;
}

/** Whether the journey may travel into this level now: on its proper picture always; on a coarse one only when it
 * went early, never for film (a capture never holds a stand-in), and only when the coarse picture fits. */
export function levelPasses(a: boolean | LevelAnswer | undefined, how: PlayHow = WAITS): boolean {
  const x = answerOf(a);
  return x.state === "proper" || (x.state === "coarse" && how.early && !how.film && coarseFits(x));
}

/**
 * What a level shows when the journey is in it: the proper picture, the coarse one, or the journey holds. A level
 * travelled into on its coarse picture keeps it until the journey is at rest (a stop or the end), then settles onto
 * the proper one (design 6: "settles at the next stop"); film never shows the coarse one.
 */
export function levelShows(a: boolean | LevelAnswer | undefined, how: PlayHow, was: "proper" | "coarse" | null, at_rest: boolean): "proper" | "coarse" | "hold" {
  const x = answerOf(a);
  const proper = x.state === "proper";
  if (how.film) return proper ? "proper" : "hold";
  if (was === "coarse") return proper ? (at_rest ? "proper" : "coarse") : coarseFits(x) ? "coarse" : "hold";
  if (proper) return "proper";
  return how.early && coarseFits(x) ? "coarse" : "hold";
}

/**
 * How far the player may go while views are still reading (design 5.4): never into the fade of a rung whose view
 * cannot be shown. The first rung that does not pass holds the journey just before its seam; when it is the first
 * rung of all, the journey holds at its start. `ready[i]` is the port's answer for `plan.path[i]` (true for proper).
 */
export function readyHold(plan: JourneyPlan, t: number, ready: readonly (boolean | LevelAnswer | undefined)[], how: PlayHow = WAITS): { t: number; waiting: number | null } {
  for (let i = 0; i < plan.path.length; i++) {
    if (levelPasses(ready[i], how)) continue;
    const e = plan.path[i]!;
    const cap = i === 0 ? 0 : Math.max(0, e.t_in - SEAM_HOLD_S);
    if (t >= cap) return { t: cap, waiting: i };
    return { t, waiting: null };
  }
  return { t, waiting: null };
}

// --- warming up the way down (FL7's journey use, design 6A) ------------------------------------------------------

/** The design's first-time estimates for a level's reads on a desktop, ms (GZ0: the web cube with its galaxies 4 to
 * 8 s, the neighbourhood 8.7 s). A device's own times replace them after one run (`BuildTimes`). */
export const JOURNEY_BUILD_MS: Partial<Record<LadderRung, number>> = { web: 6000, cluster: 8700 };
const kindOf = (r: LadderRung) => `journey.${r}`;

/** The levels still to be made ready, as the warm-up's list: each with when the journey will need it, the one on
 * screen first. */
export function journeyWarm(plan: JourneyPlan, frame: Pick<JourneyFrame, "entry"> & { t: number }, ready: readonly (boolean | LevelAnswer | undefined)[]): WarmItem[] {
  const out: WarmItem[] = [];
  plan.path.forEach((e, i) => {
    if (answerOf(ready[i]).state === "proper") return;
    out.push({ key: `journey:${i}:${e.rung}`, kind: kindOf(e.rung), needed_in_s: Math.max(0, (i === 0 ? 0 : e.t_in) - frame.t), on_screen: i === frame.entry, distance: Math.abs(i - frame.entry) });
  });
  return out;
}

/**
 * Which level may read now. The kernel answers one read at a time, so a level read out of turn only delays the one
 * needed first: the levels read one after another in the order the journey needs them, the one on screen first,
 * under the warm-up's hold-backs (a viewer who opened a journey asked for it, so a hot device still reads the next).
 * Returns the path entry, or null when every level is ready.
 */
export function journeyReadNext(plan: JourneyPlan, frame: Pick<JourneyFrame, "entry"> & { t: number }, ready: readonly (boolean | LevelAnswer | undefined)[], host: WarmHost): number | null {
  const next = warmPlan(journeyWarm(plan, frame, ready), host)[0];
  return next ? Number(next.key.split(":")[1]) : null;
}

/** The wait's one line (design 6A), "Getting the way down ready: about 15 seconds, 1 of 2 levels.", from this
 * device's times for the levels still to come and how long the one reading now has been at it. */
export function journeyWaitLine(plan: JourneyPlan, ready: readonly (boolean | LevelAnswer | undefined)[], times: Pick<BuildTimes, "estimate">, reading_ms = 0): string {
  let left = 0;
  let about = false;
  let done = 0;
  let first = true;
  plan.path.forEach((e, i) => {
    if (answerOf(ready[i]).state === "proper") return void (done += 1);
    const est = times.estimate(kindOf(e.rung), JOURNEY_BUILD_MS[e.rung] ?? 2000);
    about ||= est.about;
    left += first ? Math.max(500, est.ms - reading_ms) : est.ms;
    first = false;
  });
  return readyLine("the way down", done, plan.path.length, left, about);
}

/** The start level's slow turn while the way down gets ready (design 6A: something worth watching), radians about the
 * page's up, as the web cube's own screen turns. Once the journey goes, the turn eases back to the journey's line of
 * sight within half a second, so the camera the plan gives is the one drawn by the time anything else is. */
export const START_TURN_RAD_S = 0.12;
export function startTurn(angle: number, dt_s: number, turning: boolean): number {
  if (turning) return angle + START_TURN_RAD_S * dt_s;
  const w = Math.atan2(Math.sin(angle), Math.cos(angle));
  const back = w * Math.max(0, 1 - dt_s / 0.07);
  return Math.abs(back) < 1e-3 ? 0 : back;
}

/** A line of sight turned about the page's up (z), for the start level's slow turn. */
export function turnedAboutUp(d: Vec3, angle: number): Vec3 {
  const c = Math.cos(angle);
  const s = Math.sin(angle);
  return [c * d[0] - s * d[1], s * d[0] + c * d[1], d[2]];
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
