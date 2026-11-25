// Pencil and ink (track SV, SV3: the shared draw style). On the GM's chart what the table knows is drawn in ink and
// what only the GM knows in pencil: a quiet, dotted version of the same mark, line or name. "Heard of" (known to
// exist, details held back) is a thin solid outline with no fill. No icons are added and no colour carries the
// meaning, so it reads in the Night chart, in one-colour themes and for every colour vision. Players never see
// pencil: their screens draw only what they know, in ink. Every chart takes its look from here, so the same secret
// reads the same on every chart.
import type { KnownLandmark as Landmark } from "../../ui-contract/src/index.ts";

export type Pen = "ink" | "heard" | "pencil";

/** How a thing the table knows to this landmark is drawn on the GM's chart. */
export function penOf(landmark: Landmark | null | undefined): Pen {
  return landmark === "gm_only" ? "pencil" : landmark === "heard_of" ? "heard" : "ink";
}

/** The words a pencilled or heard-of thing carries for a screen reader and on hover. */
export const PEN_WORDS: Record<Pen, string> = { ink: "", heard: "the table has heard of it", pencil: "not known to the table" };

/**
 * Inline styles for a pen, so any chart (SVG or HTML) draws it alike without sharing a stylesheet:
 * - `mark`: a body, star or construct's mark (pencil: faint; heard: outline only);
 * - `line`: an orbit, link, route or border (pencil: dotted and muted; heard: thin solid muted);
 * - `name`: its name, in SVG text or HTML (pencil: muted italic with a dotted underline; heard: muted).
 */
export function penStyle(pen: Pen, what: "mark" | "line" | "name"): string {
  if (pen === "ink") return "";
  if (what === "mark") return pen === "pencil" ? "opacity: 0.4" : "fill-opacity: 0; opacity: 0.75";
  if (what === "line") return pen === "pencil" ? "stroke: var(--sse-ink-muted); stroke-dasharray: 1.5 2.5" : "stroke: var(--sse-ink-muted); stroke-width: 0.8px";
  return pen === "pencil"
    ? "fill: var(--sse-ink-muted); color: var(--sse-ink-muted); font-style: italic; text-decoration: underline dotted; text-underline-offset: 3px"
    : "fill: var(--sse-ink-muted); color: var(--sse-ink-muted)";
}

/** The ring drawn round a pencilled or heard-of mark: dotted for pencil, thin solid for heard of (none for ink). */
export function penRing(pen: Pen): { dash: string | null; width: number } | null {
  return pen === "pencil" ? { dash: "1.5 2.5", width: 0.8 } : pen === "heard" ? { dash: null, width: 0.8 } : null;
}
