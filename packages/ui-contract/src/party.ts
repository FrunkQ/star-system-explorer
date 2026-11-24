// The party's ship (landing run-through, blocker 4): the one construct the
// table travels in, carried from Ships and stations to Voyages to Land. The GM
// picks it; Voyages starts where it is and moves it on arrival; Land offers it
// as the lander, flown on its own numbers (plan_descent `{ship}`). Kept with
// the campaign save. Where it is will move to the construct's flight log in
// the record; until then `at` is the table's own note of it.
import type { Json } from "./json.ts";
import type { NavShip, ShipChoice } from "./route.ts";

export interface PartyShip {
  /** The construct's address. */
  ship: string;
  /** Its name, as the table calls it. */
  name: string;
  /** Its nav ship: a stock id or the pack's ship object, as Voyages sends it. */
  nav: string | NavShip;
  /** The body (or system) it is at now. */
  at: string;
  /** The last voyage flown, for the players' screens. */
  voyage?: { from: string; to: string; days: number; depart_yr?: number };
}

/** The nav ship a construct names (`construct.nav_ship`), among the picker's rows; null when it names none known. */
export function navShipOf(navShip: unknown, choices: readonly ShipChoice[]): string | NavShip | null {
  if (typeof navShip !== "string") return null;
  const c = choices.find((x) => x.id === navShip);
  return c ? (c.ship ?? c.id) : null;
}

/** The lander to send plan_descent for the party's ship. */
export function partyLander(p: PartyShip): Json {
  return { ship: p.nav as Json };
}

/**
 * Where a voyage starts: the party's ship if it is in this system, else the
 * GM's selected world if it is one of the places, else the fallback.
 */
export function voyageStart(places: readonly string[], party: PartyShip | null | undefined, selection: string | null | undefined, fallback: string): string {
  if (party && places.includes(party.at)) return party.at;
  if (selection) {
    // A moon or a construct picks the world it belongs to.
    const hit = places.filter((p) => selection === p || selection.startsWith(`${p}/`)).sort((a, b) => b.length - a.length)[0];
    if (hit) return hit;
  }
  return fallback;
}
