// The stepper (three desks design 3.2, TM7): on the Run desk the place line is the main way to move. Every level is
// drawn, whether or not the GM has been there; the one in view is in the accent; levels below it that the trail
// remembers are pencilled with their names, the rest read as the level's word. Out and In sit at the line's end.
// Presentation only, like the place line it draws: a step is a Place, and the shell's goPlace travels to it.
// Travel is a plain swap until the grand zoom's short form (GZ4) lands. Local stars and Site join PLACE_SCALES with GZ1;
// the stepper draws whatever PLACE_SCALES holds, so they appear with no change here.
import { PLACE_SCALES, PLACE_WORDS, placeRank, type Place, type PlaceScale, type PlaceStep } from "./place.ts";

/** One step of the stepper: a place-line step, or a level not visited yet (no place: a press goes In towards it). */
export interface StepperStep {
  scale: PlaceScale;
  label: string;
  place: Place | null;
  current: boolean;
  below: boolean;
  /** A level below the one in view that the trail does not reach: drawn as the level's word. */
  unvisited: boolean;
}

/** Every level, largest first: the place line's own steps where it has them, the level's word elsewhere. */
export function stepperLine(steps: readonly PlaceStep[]): StepperStep[] {
  return PLACE_SCALES.map((scale) => {
    const s = steps.find((x) => x.scale === scale);
    return s ? { ...s, unvisited: false } : { scale, label: PLACE_WORDS[scale], place: null, current: false, below: true, unvisited: true };
  });
}

/** Out: the level above the one in view, or null at the top. */
export function stepOut(steps: readonly PlaceStep[]): Place | null {
  const i = steps.findIndex((x) => x.current);
  return i > 0 ? steps[i - 1]!.place : null;
}

/**
 * In: down one level, onto the target the trail remembers, else onto the thing selected (a body, at a system), else
 * onto the level's way in (the galaxies nearby from the web, the galaxy from there, the campaign's system from the
 * galaxy). Null when there is nothing to go into (a system with nothing selected, a world until Site exists).
 */
export function stepIn(steps: readonly PlaceStep[], here: Place, selection: string | null): Place | null {
  const next = PLACE_SCALES[placeRank(here.scale) + 1];
  if (!next) return null;
  const remembered = steps.find((x) => x.scale === next && x.below);
  if (remembered) return remembered.place;
  if (next === "world") return selection ? { scale: "world", system: here.system, world: selection } : null;
  return { scale: next, system: here.system };
}
