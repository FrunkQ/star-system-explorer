// The party's ship (landing run-through, blocker 4): the one construct the
// table travels in, carried from Ships and stations to Voyages to Land. The GM
// picks it; Voyages starts where it is and moves it on arrival; Land offers it
// as the lander, flown on its own numbers (plan_descent `{ship}`). Kept with
// the campaign save. Where it is will move to the construct's flight log in
// the record; until then `at` is the table's own note of it.
import type { Json } from "./json.ts";
import { constant } from "./sheet.ts";
import { FUEL_FIELDS, fuelNow, shipRecordOf, tonnesWords, type FuelShip } from "./manifest.ts";
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

const DAY_S = constant("day");

/** One entry of a construct's flight log, as `log_flight` takes it (without the construct and the writing time). */
export type FlightEntryAsk = { piece: Json } | { event: Json };

/**
 * A flown voyage as flight-log entries (the record's `log_flight`): leaving,
 * each leg as planned, arriving, and parked in a low orbit at the end (300 km
 * up, an assumption said in its `why`). Marked `planned`: the table flew it,
 * nothing recorded it. Legs run back to back from the plan's departure; each
 * leg's days include its wait for a window. A belt has no orbit to park in.
 */
export function voyageLogEntries(
  plan: { depart_s?: number; voyage: { from: string; to: string; legs: readonly { from: string; to: string; home_days: number; means?: string }[] } | null },
  bodies: readonly { address: string; radius_m: number }[],
  depart_s = 0,
): FlightEntryAsk[] {
  const v = plan.voyage;
  if (!v || !v.legs.length) return [];
  const mark = "planned";
  let t = Math.round(plan.depart_s ?? depart_s);
  const out: FlightEntryAsk[] = [{ event: { at_s: t, kind: "stop", type: "depart", place_id: v.from, mark } }];
  for (const l of v.legs) {
    const end = t + Math.round(l.home_days * DAY_S);
    // A leg between the stars (a jump or a flight) is its own kind of piece; one inside a system is a course.
    const state: Json = l.means === "jump" || l.means === "flight" ? { kind: "between_stars", leg: l as unknown as Json } : { kind: "course", course: { kind: "planned", course: l as unknown as Json } };
    out.push({ piece: { from_s: t, to_s: end, state, mark } });
    t = end;
  }
  out.push({ event: { at_s: t, kind: "stop", type: "arrive", place_id: v.to, mark } });
  const b = bodies.find((x) => x.address === v.to);
  if (b && !v.to.startsWith("belt:")) {
    out.push({ piece: { from_s: t, state: { kind: "orbit", around: v.to, elements: { a_m: b.radius_m + 300e3, e: 0, epoch_s: t } }, mark, why: "parked in a low orbit on arrival, 300 km up (assumed)" } });
  }
  return out;
}

/** The system a place is in: its address up to the `sys:` part (a system's own address is its own). */
export function systemOfPlace(address: string): string | null {
  const m = /^(.*\/sys:[^/]+)/.exec(address);
  return m ? m[1]! : null;
}

/** The ship-picker rows with the party's ship first, when the campaign has one (Voyages, in a system and between stars). */
export const PARTY_CHOICE = "party";
export function withPartyChoice<R extends ShipChoice>(rows: R[], party: PartyShip | null): (R | ShipChoice)[] {
  if (!party) return rows;
  const like = typeof party.nav === "string" ? rows.find((r) => r.id === party.nav) : undefined;
  const drive = like?.drive ?? (typeof party.nav === "string" ? "" : rows.find((r) => r.ship === party.nav)?.drive ?? `${party.nav.accel_g} g`);
  return [{ id: PARTY_CHOICE, name: `${party.name} (the party's ship)`, drive, ...(typeof party.nav === "string" ? {} : { ship: party.nav }) } as ShipChoice, ...rows];
}

/**
 * The party's ship as it is now, for planning (fuel gauges, MC4): its own numbers from the kernel with the tanks at the
 * levels kept on its record (full when never set). Null when the kernel cannot resolve it.
 */
export async function partyShipNow(
  session: Parameters<typeof shipRecordOf>[0] & { record?: (a: string) => Promise<{ ok: true; record: { kept: { kind: string; value: Json }[] } } | { ok: false }> },
  party: PartyShip,
): Promise<FuelShip | null> {
  const [r, own] = await Promise.all([shipRecordOf(session, party.nav), session.record ? session.record(party.ship) : Promise.resolve({ ok: false as const })]);
  if (!r.ok) return null;
  const kept: Record<string, unknown> = own.ok ? Object.fromEntries(own.record.kept.map((k) => [k.kind, k.value])) : {};
  return fuelNow(r.ship as unknown as FuelShip, (k) => kept[k]);
}

/**
 * What a flown voyage leaves in the party ship's tanks, as kept-value writes (fuel draw-down, Alex 24 Nov): the last
 * leg's `propellant_left_t` and `jump_fuel_left_t`, which already count every refuel on the way. Only the tanks the
 * ship has (its gauges); each says where it went, for the ship's log ("Jump fuel: 0 (−19.8) · voyage to Regina").
 */
export function voyageFuelWrites(
  voyage: { legs: readonly { propellant_left_t: number; jump_fuel_left_t: number }[] } | null,
  tanks: readonly ("propellant" | "jump")[],
  toName: string,
): { kind: string; value: number; reason: string }[] {
  const last = voyage?.legs[voyage.legs.length - 1];
  if (!last) return [];
  const reason = `voyage to ${toName}`;
  const out: { kind: string; value: number; reason: string }[] = [];
  if (tanks.includes("propellant") && Number.isFinite(last.propellant_left_t)) out.push({ kind: FUEL_FIELDS.propellant, value: Math.max(0, last.propellant_left_t), reason });
  if (tanks.includes("jump") && Number.isFinite(last.jump_fuel_left_t)) out.push({ kind: FUEL_FIELDS.jump, value: Math.max(0, last.jump_fuel_left_t), reason });
  return out;
}

/** What the tanks hold after a draw-down, in words for the voyage panel ("Tanks now: reaction mass 25.6 t, jump fuel 40.0 t."). */
export function tanksWords(writes: readonly { kind: string; value: number }[]): string | null {
  if (!writes.length) return null;
  return `Tanks now: ${writes.map((w) => `${w.kind === FUEL_FIELDS.jump ? "jump fuel" : "reaction mass"} ${tonnesWords(w.value)}`).join(", ")}.`;
}
