// Reveal on arrival (landing run-through, gap 6): when the party's ship comes
// to a world the table does not yet fully know, the GM is offered one press to
// tell them. Guide, don't decide: nothing is revealed by itself, and the offer
// goes away if the GM ignores it. The press is the ordinary `set_known` intent
// (kernel SV1), so it is one dated event with the usual Undo. Sensor-driven
// reveals (RK14) will write the same grant later.
import type { PartyShip } from "./party.ts";
import type { Intent } from "./intent.ts";
import type { KnownAudience } from "./known.ts";

export interface ArrivalOffer {
  /** The world the party has come to. */
  world: string;
  /** The words on the slip: "Beowulf is at Mars. Tell the table about it?" */
  line: string;
  /** The press: "Tell the table". */
  action: string;
}

/**
 * The offer to make when the party's ship moves from `before` to `after`, or
 * null: none when it has not moved to a new place, when the place is a whole
 * system, or when the table already knows the world (`table` is its
 * Known to row for the table, null when the kernel gave none).
 */
export function arrivalOffer(before: PartyShip | null | undefined, after: PartyShip | null | undefined, worldName: string, table: KnownAudience | null | undefined): ArrivalOffer | null {
  if (!after || !after.at) return null;
  if (before && before.ship === after.ship && before.at === after.at) return null;
  if (!/\/body:/.test(after.at)) return null;
  if (table && table.landmark === "known" && table.top) return null;
  const name = worldName.trim() || "this world";
  return { world: after.at, line: `${after.name} is at ${name}. Tell the table about it?`, action: "Tell the table" };
}

/** The `set_known` intent the offer's press sends: the world and what is on it, known to the table's audience. */
export function arrivalReveal(offer: ArrivalOffer, audience: string, at: { at_yr: number; at_s?: string }): Intent {
  return { kind: "set_known", address: offer.world, audience, landmark: "known", scope: "this", at_yr: at.at_yr, ...(at.at_s !== undefined ? { at_s: at.at_s } : {}) };
}
