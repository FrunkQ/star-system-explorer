// One star's life (SC1 follow-up, Alex 26 Nov: "see these stars on a timeline... order by 'same star'... animating
// between them... and have earth at 1au to show scale"). A starting mass's stages in order, each read off the kernel's
// life line (`describe_star`, K18): the stages and their ages are the life line's own, whatever the mass (a red dwarf
// that never swells, a massive star's Wolf-Rayet stage when the life line has one), never a list typed here. Each stage
// is a run of frames, the star as the life line gives it at ages through the stage, each with its own surface from the
// kernel's activity, so playing the life redraws the star as it is at each age, not a blend of pictures.
import { constant, type StarDescription, type StarPhase } from "../../../ui-contract/src/index.ts";
import type { LiveStar } from "../../../renderer/src/index.ts";
import { NOT_DRAWN, readSurface, slug, type Reader } from "./star-gallery.ts";

/** The star at one age of its life. */
export interface LifeFrame {
  age_yr: number;
  now: StarDescription["now"];
  /** The living star to draw; null where it is not drawn yet. */
  live: LiveStar | null;
  /** Its own clock, star seconds per screen second (its cells or its turn). */
  clock: number;
}

/** One stage of the life: the life line's phase, its ages, and the star through it. */
export interface LifeStage {
  phase: StarPhase;
  label: string;
  from_yr: number;
  /** Null for the stage it ends in, which lasts. */
  to_yr: number | null;
  frames: LifeFrame[];
  /** Why the stage is not drawn, for a remnant still to be drawn by the exotic objects renderer. */
  why?: string;
}

export interface StarLife {
  mass_msun: number;
  feh: number;
  stages: LifeStage[];
  remnant: StarDescription["remnant"];
  notes: string[];
}

/** Frames read through each stage. */
export const FRAMES_PER_STAGE = 10;
/** The span shown of the stage a star ends in, which has no end: its first ten billion years. */
export const LAST_STAGE_SHOWN_YR = 1e10;
/** The first age shown of a stage that lasts, years after it starts (a white dwarf's first million years are its hottest). */
const LAST_STAGE_FIRST_YR = 1e6;

/**
 * The ages a stage is read at. A stage with an end: spread through it, closer together towards its end, where a star
 * changes fastest (a giant climbing to its tip). The stage it ends in: spread in log time over its first ten billion
 * years. Never at a stage's very edge, where the life line hands over to the next.
 */
export function stageAges(from_yr: number, to_yr: number | null, n = FRAMES_PER_STAGE): number[] {
  const out: number[] = [];
  for (let i = 0; i < n; i++) {
    const u = (i + 0.5) / n;
    if (to_yr === null) out.push(from_yr + LAST_STAGE_FIRST_YR * (LAST_STAGE_SHOWN_YR / LAST_STAGE_FIRST_YR) ** u);
    else out.push(from_yr + (to_yr - from_yr) * (1 - (1 - u) ** 2));
  }
  return out;
}

/** Read a whole life: the stages of a star born with `mass_msun`, each with its frames. `onstage` hears each as it is read. */
export async function readLife(session: Reader, mass_msun: number, feh?: number, onstage?: (s: LifeStage, i: number) => void): Promise<StarLife | string> {
  if (!session.describeStar) return "this seat cannot read the life line";
  const describe = session.describeStar.bind(session);
  const first = await describe(mass_msun, 1e6, feh);
  if (!first.ok) return first.message;
  const d = first.star;
  const address = `u:star-gallery/star:life-${slug(String(mass_msun))}`;
  const stages: LifeStage[] = [];
  for (const [i, p] of d.phases.entries()) {
    const stage: LifeStage = { phase: p.phase, label: p.label, from_yr: p.from_yr, to_yr: p.to_yr, frames: [] };
    const notYet = NOT_DRAWN[p.phase];
    if (notYet) stage.why = notYet;
    // A stage's ages asked at once, so they queue at the kernel together rather than one at a time.
    const frames = await Promise.all(
      stageAges(p.from_yr, p.to_yr).map(async (age_yr): Promise<LifeFrame | string> => {
        const r = await describe(mass_msun, age_yr, feh);
        if (!r.ok) return r.message;
        const now = r.star.now;
        let live: LiveStar | null = null;
        let clock = 1;
        if (!notYet && now.teff_k > 0 && now.radius_m > 0 && now.mass_kg > 0) {
          const s = await readSurface(session, address, now, age_yr);
          if (typeof s !== "string") ((live = s.live), (clock = s.clock));
        }
        return { age_yr, now, live, clock };
      }),
    );
    const failed = frames.find((f): f is string => typeof f === "string");
    if (failed) return failed;
    stage.frames = frames as LifeFrame[];
    stages.push(stage);
    onstage?.(stage, i);
  }
  return { mass_msun, feh: d.feh, stages, remnant: d.remnant, notes: d.notes };
}

/** Where the playhead is: a stage, a frame in it, and how far between that frame and the next (0..1). */
export function frameAt(life: Pick<StarLife, "stages">, play: number): { stage: number; frame: number; between: number } {
  const n = life.stages.length;
  const p = Math.max(0, Math.min(n - 1e-9, play));
  const stage = Math.floor(p);
  const k = life.stages[stage]!.frames.length;
  const f = (p - stage) * k;
  const frame = Math.min(k - 1, Math.floor(f));
  return { stage, frame, between: f - frame };
}

/** The star's radius at the playhead, metres: between a stage's frames in log, so a giant swells smoothly rather than in steps. */
export function radiusAt(life: Pick<StarLife, "stages">, play: number): number {
  const { stage, frame, between } = frameAt(life, play);
  const frames = life.stages[stage]!.frames;
  const a = frames[frame]!.now.radius_m;
  // Within a stage only: a stage's end hands over at once (a giant's envelope gone, a white dwarf left), never a blend.
  const b = frames[frame + 1]?.now.radius_m ?? a;
  if (!(a > 0 && b > 0)) return a;
  return a * (b / a) ** between;
}

/** The largest the star is over its life, metres. */
export const largestRadius = (life: Pick<StarLife, "stages">): number => Math.max(0, ...life.stages.flatMap((s) => s.frames.map((f) => f.now.radius_m)));

/** Earth's orbit, metres. */
export const AU = constant("au");

/** Pixels per metre for a scene `fit_px` wide from the centre: the whole life (its largest size, or Earth's orbit if larger), or Earth's orbit. */
export function sceneScale(life: Pick<StarLife, "stages">, fit_px: number, fit: "life" | "earth"): number {
  const reach = fit === "earth" ? AU : Math.max(AU, largestRadius(life));
  return fit_px / reach;
}
