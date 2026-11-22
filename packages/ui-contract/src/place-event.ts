// What a scale's own view says about where it is (U24's place line, V9's maps).
// A view that has places of its own (a cell of galaxies, a galaxy, a slice of
// the cosmic web) tells the shell which one is in view, and what is nested
// below it, by a bubbling "sse:place" event; and it opens on the place a link
// names through its `place` param. The ids are the views' own: a cell is
// `cell:x.y.z`, a galaxy is the kernel's halo address (or a named reference
// halo's key), a slice of the web is `web:<extent in Mpc>`. Presentation only.

import type { PlaceScale } from "./place.ts";

export const PLACE_EVENT = "sse:place";

/** One place below the one in view, for the place line's Below list. */
export interface PlaceChild {
  id: string;
  name: string;
}

/** The detail of an "sse:place" event. */
export interface PlaceNote {
  scale: PlaceScale;
  id: string;
  name: string;
  children?: PlaceChild[];
}

/** Tell the shell where a view is. Does nothing without an element (before mount). */
export function announcePlace(el: Element | null | undefined, note: PlaceNote): void {
  el?.dispatchEvent(new CustomEvent<PlaceNote>(PLACE_EVENT, { bubbles: true, detail: note }));
}

/** A halo cell's id. */
export function cellId(c: readonly [number, number, number]): string {
  return `cell:${c[0]}.${c[1]}.${c[2]}`;
}

/** The cell a place id names: a cell id, or any address with a cell in it (a halo's). */
export function parseCellId(id: unknown): [number, number, number] | null {
  const m = typeof id === "string" ? /(?:^|\/)cell:(-?\d+)\.(-?\d+)\.(-?\d+)(?:\/|$)/.exec(id) : null;
  return m ? [Number(m[1]), Number(m[2]), Number(m[3])] : null;
}

/** A slice of the cosmic web's id: its side in megaparsecs. */
export function webId(extent_mpc: number | string): string {
  return `web:${extent_mpc}`;
}

/** The side a web id names, or null. */
export function parseWebId(id: unknown): string | null {
  const m = typeof id === "string" ? /^web:(\d+(?:\.\d+)?)$/.exec(id) : null;
  return m ? m[1]! : null;
}
