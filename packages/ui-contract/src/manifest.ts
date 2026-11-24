// The ship manifest and crew roster (track MC): who and what is aboard a ship, station or site. Everyone and
// everything aboard is a record actor (kind `person` or `cargo`) with an `aboard.on` link to where it is, kept by the
// base manifest pack (`sse.base-manifest`). Moving someone is one dated change; the history keeps where they were.
// Nothing here names a game: roles, kinds of being and goods are words, and the life support rates are pack data
// per kind of being, so a kind with no rates says so rather than borrowing a human's.
import type { Json } from "./json.ts";
import type { KeptChange, RecordRead } from "./constructs.ts";
import { YEAR_S } from "./intent.ts";

/** The kept kinds the manifest reads and writes. */
export const ABOARD = {
  on: "aboard.on",
  role: "aboard.role",
  state: "aboard.state",
  being: "aboard.being",
  age: "aboard.age_yr",
  what: "aboard.what",
  amount: "aboard.amount",
  unit: "aboard.unit",
  recycled: "life_support.recycled",
} as const;

/** The two kinds of entry. Any other actor kind with an `aboard.on` link is listed with the cargo, by its own kind. */
export const PERSON = "person";
export const CARGO = "cargo";

export const STATES = ["awake", "cold_sleep", "missing", "dead"] as const;
export type AboardState = (typeof STATES)[number];
export const STATE_WORDS: Record<AboardState, string> = { awake: "awake", cold_sleep: "in cold sleep", missing: "missing", dead: "dead" };

/** One person or lot, read from the record. Times are campaign seconds. */
export interface AboardEntry {
  address: string;
  name: string;
  kind: string;
  on?: string;
  role?: string;
  state: AboardState;
  being?: string;
  /** Own age when it was last set, and when. */
  age_yr?: number;
  age_set_s?: number;
  what?: string;
  amount?: number;
  unit?: string;
  /** Every change to `aboard.state` and `aboard.on`, oldest first, for ageing and the log. */
  states: KeptChange[];
  moves: KeptChange[];
}

const text = (v: Json | null | undefined): string | undefined => (typeof v === "string" && v.trim() ? v : undefined);
const num = (v: Json | null | undefined): number | undefined => (typeof v === "number" && Number.isFinite(v) ? v : undefined);

/** An entry from an actor and its record. */
export function entryFrom(actor: { address: string; name: string; kind: string }, rec: RecordRead | null): AboardEntry {
  const k = (kind: string) => rec?.kept.find((x) => x.kind === kind);
  const state = text(k(ABOARD.state)?.value);
  const age = k(ABOARD.age);
  const last = age?.history[age.history.length - 1];
  return {
    address: actor.address,
    name: actor.name,
    kind: actor.kind,
    ...(text(k(ABOARD.on)?.value) ? { on: text(k(ABOARD.on)?.value)! } : {}),
    ...(text(k(ABOARD.role)?.value) ? { role: text(k(ABOARD.role)?.value)! } : {}),
    state: (STATES as readonly string[]).includes(state ?? "") ? (state as AboardState) : "awake",
    ...(text(k(ABOARD.being)?.value) ? { being: text(k(ABOARD.being)?.value)! } : {}),
    ...(num(age?.value) !== undefined ? { age_yr: num(age?.value)!, age_set_s: last?.at_s ?? 0 } : {}),
    ...(text(k(ABOARD.what)?.value) ? { what: text(k(ABOARD.what)?.value)! } : {}),
    ...(num(k(ABOARD.amount)?.value) !== undefined ? { amount: num(k(ABOARD.amount)?.value)! } : {}),
    ...(text(k(ABOARD.unit)?.value) ? { unit: text(k(ABOARD.unit)?.value)! } : {}),
    states: k(ABOARD.state)?.history ?? [],
    moves: k(ABOARD.on)?.history ?? [],
  };
}

/** Who and what is aboard `place` now: people and cargo apart, each by name. */
export function aboard(entries: readonly AboardEntry[], place: string): { people: AboardEntry[]; cargo: AboardEntry[] } {
  const here = entries.filter((e) => e.on === place);
  const by = (a: string, b: string) => (a < b ? -1 : a > b ? 1 : 0);
  return {
    people: here.filter((e) => e.kind === PERSON).sort((a, b) => by(a.name, b.name)),
    cargo: here.filter((e) => e.kind !== PERSON).sort((a, b) => by(a.name, b.name)),
  };
}

/** Does this person draw on the stores and grow older? Awake and missing people do (missing is not known dead). */
export function living(s: AboardState): boolean {
  return s === "awake" || s === "missing";
}

/**
 * A person's own age at `now_s`: the age last set, plus the campaign time since then that they spent awake (or
 * missing). Cold sleep and death stop the clock. Fudge register: cold sleep stops ageing entirely, and time dilation
 * on fast voyages is not applied yet (it joins when voyages record ship time).
 */
export function ageNow(e: AboardEntry, now_s: number): number | undefined {
  if (e.age_yr === undefined) return undefined;
  const from = e.age_set_s ?? 0;
  if (now_s <= from) return e.age_yr;
  // The state in force at `from`, then each change after it.
  let state: AboardState = "awake";
  for (const c of e.states) if (c.at_s <= from && typeof c.value === "string") state = c.value as AboardState;
  let t = from;
  let lived = 0;
  for (const c of e.states) {
    if (c.at_s <= from || c.at_s > now_s) continue;
    if (living(state)) lived += c.at_s - t;
    t = c.at_s;
    if (typeof c.value === "string") state = c.value as AboardState;
  }
  if (living(state)) lived += now_s - t;
  return e.age_yr + lived / Number(YEAR_S);
}

// ---- Life support ----

export interface LifeSupportRate {
  oxygen_kg_day?: number;
  water_kg_day?: number;
  food_kg_day?: number;
  recyclable?: string[];
  why?: string;
  src?: string;
  check?: string;
}
export interface LifeSupportPack {
  rates: Record<string, LifeSupportRate>;
  store_words: Record<string, string[]>;
  sources?: Record<string, string>;
}
export const STORES = ["oxygen", "water", "food"] as const;
export type Store = (typeof STORES)[number];

/** Read a pack's `provides.other.life_support`, top of the stack winning per kind of being. */
export function lifeSupportFrom(packs: readonly Json[]): LifeSupportPack {
  const out: LifeSupportPack = { rates: {}, store_words: {}, sources: {} };
  for (const p of packs) {
    const ls = (p as { provides?: { other?: { life_support?: Partial<LifeSupportPack> } } })?.provides?.other?.life_support;
    if (!ls) continue;
    Object.assign(out.rates, ls.rates ?? {});
    Object.assign(out.store_words, ls.store_words ?? {});
    Object.assign(out.sources!, ls.sources ?? {});
  }
  return out;
}

/** Which store a lot is, from its `aboard.what` and the pack's words; none for ordinary goods. */
export function storeOf(what: string | undefined, words: Record<string, string[]>): Store | undefined {
  const w = (what ?? "").trim().toLowerCase();
  if (!w) return undefined;
  return STORES.find((s) => s === w || (words[s] ?? []).includes(w));
}

/** A lot's mass in kg, when its unit is a mass the manifest knows (kg or t); none for counts and volumes. */
export function massKg(amount: number | undefined, unit: string | undefined): number | undefined {
  if (amount === undefined) return undefined;
  const u = (unit ?? "").trim().toLowerCase();
  if (u === "kg") return amount;
  if (u === "t" || u === "tonne" || u === "tonnes") return amount * 1000;
  return undefined;
}

export interface StoreDays {
  store: Store;
  kg: number;
  /** Net use a day by everyone aboard who draws on it, after recycling. */
  use_kg_day: number;
  /** Days left; null when nobody aboard uses it at a known rate. */
  days: number | null;
}
export interface LifeSupport {
  stores: StoreDays[];
  /** The store that runs out first, and when. */
  first?: { store: Store; days: number };
  /** People drawing on the stores, and those whose kind of being has no rates (named by kind). */
  drawing: number;
  unrated: { being: string; count: number }[];
  /** Lots of a store whose unit is not a mass (left out of the sums). */
  unweighed: string[];
}

/**
 * Days of life support aboard: each store's mass over the net daily use of everyone living aboard (awake or
 * missing), at their kind of being's rates, less the recycled share for stores the rates mark recyclable. A person
 * whose kind of being is not stated is taken as `defaultBeing` (the screen says so); a kind with no rates is counted
 * apart and named, never given another kind's.
 */
export function lifeSupport(people: readonly AboardEntry[], cargo: readonly AboardEntry[], ls: LifeSupportPack, recycled = 0, defaultBeing = "human"): LifeSupport {
  const share = Math.min(1, Math.max(0, recycled));
  const use: Record<Store, number> = { oxygen: 0, water: 0, food: 0 };
  const unrated = new Map<string, number>();
  let drawing = 0;
  for (const p of people) {
    if (!living(p.state)) continue;
    const being = p.being ?? defaultBeing;
    const r = ls.rates[being.toLowerCase()];
    if (!r) {
      unrated.set(being, (unrated.get(being) ?? 0) + 1);
      continue;
    }
    drawing++;
    for (const s of STORES) {
      const day = r[`${s}_kg_day` as const] ?? 0;
      use[s] += (r.recyclable ?? []).includes(s) ? day * (1 - share) : day;
    }
  }
  const kg: Record<Store, number> = { oxygen: 0, water: 0, food: 0 };
  const unweighed: string[] = [];
  for (const c of cargo) {
    const s = storeOf(c.what ?? c.name, ls.store_words);
    if (!s) continue;
    const m = massKg(c.amount, c.unit);
    if (m === undefined) unweighed.push(c.name);
    else kg[s] += m;
  }
  const stores = STORES.map((s) => ({ store: s, kg: kg[s], use_kg_day: use[s], days: use[s] > 0 ? kg[s] / use[s] : null }));
  const timed = stores.filter((s) => s.days !== null).sort((a, b) => a.days! - b.days!);
  return {
    stores,
    ...(timed.length ? { first: { store: timed[0]!.store, days: timed[0]!.days! } } : {}),
    drawing,
    unrated: [...unrated].map(([being, count]) => ({ being, count })).sort((a, b) => (a.being < b.being ? -1 : 1)),
    unweighed,
  };
}

// ---- New entries ----

/** A record id for a new entry from its name, unique among `taken` addresses ("Ada Moss" is `ada-moss`, then `ada-moss-2`). */
export function entryId(name: string, taken: readonly string[]): string {
  const base = name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 40) || "entry";
  const ids = new Set(taken.map((a) => a.split("/").pop()?.replace(/^actor:/, "") ?? ""));
  if (!ids.has(base)) return base;
  let n = 2;
  while (ids.has(`${base}-${n}`)) n++;
  return `${base}-${n}`;
}

/** The kept values a new person or lot starts with, as kind and value pairs, blanks left out. */
export function startingValues(kind: string, on: string, f: { role?: string; being?: string; age_yr?: number; state?: AboardState; what?: string; amount?: number; unit?: string }): [string, Json][] {
  const out: [string, Json][] = [[ABOARD.on, on]];
  const t = (k: string, v: string | undefined) => v?.trim() && out.push([k, v.trim()]);
  if (kind === PERSON) {
    t(ABOARD.role, f.role);
    t(ABOARD.being, f.being);
    if (f.age_yr !== undefined && Number.isFinite(f.age_yr)) out.push([ABOARD.age, f.age_yr]);
    if (f.state && f.state !== "awake") out.push([ABOARD.state, f.state]);
  } else {
    t(ABOARD.what, f.what);
    if (f.amount !== undefined && Number.isFinite(f.amount)) out.push([ABOARD.amount, f.amount]);
    t(ABOARD.unit, f.unit);
  }
  return out;
}

/**
 * How many people are aboard a construct, the one number every screen reads (the card, the ship's log, the "At
 * world" card): the roster's count of people aboard and not dead or missing when the ship has a roster, else the
 * template's starting figure `construct.crew`. The templates' complement and berths (e.g. `traveller.crew`,
 * `traveller.berths`) are design figures, not people aboard, and are never read here.
 */
export function crewAboard(subject: { address: string; facts: readonly { field: string; value?: unknown }[] }, roster: readonly AboardEntry[] | null): number | undefined {
  const here = roster ? aboard(roster, subject.address).people : [];
  if (here.length) return here.filter((p) => p.state !== "dead" && p.state !== "missing").length;
  const v = subject.facts.find((f) => f.field === "construct.crew")?.value;
  return typeof v === "number" ? v : undefined;
}

// ---- Fuel aboard (MC4, Alex 24 Nov: "fuel gauge (and an edit fuel) to top up"): how full each tank is now ----
// Facts on the construct, changed in play like what is aboard. The kernel's ship already reads both ("propellant now,
// default full"; a jump drive's "fuel aboard now"), so a voyage planned from `fuelNow` starts from what is in the tanks.

export const FUEL_FIELDS = { propellant: "construct.propellant_t", jump: "construct.jump_fuel_t" } as const;
export type FuelTank = keyof typeof FUEL_FIELDS;

export interface FuelGauge {
  tank: FuelTank;
  /** What is in it, for the label ("deuterium", "hydrogen"). */
  fuel: string;
  full_t: number;
  now_t: number;
  /** now / full, 0 to 1. */
  share: number;
}

/** The ship's numbers this needs: the kernel's ship (nav) record. */
export interface FuelShip {
  wet_t: number;
  dry_t: number;
  propellant_t?: number;
  fuels?: string[];
  jump?: { tank_t: number; fuel_t?: number; fuel?: string } | null;
}

const level = (v: unknown, full: number) => (typeof v === "number" && Number.isFinite(v) ? Math.max(0, Math.min(full, v)) : full);

/** A gauge per tank the ship has (reaction mass, and jump fuel when it has a jump drive); kept levels, else full. */
export function fuelGauges(ship: FuelShip, kept: (field: string) => unknown): FuelGauge[] {
  const out: FuelGauge[] = [];
  const full = Math.max(0, ship.wet_t - ship.dry_t);
  if (full > 0) {
    const now = level(kept(FUEL_FIELDS.propellant), full);
    out.push({ tank: "propellant", fuel: ship.fuels?.join(" and ") || "propellant", full_t: full, now_t: now, share: now / full });
  }
  const j = ship.jump;
  if (j && j.tank_t > 0) {
    const now = level(kept(FUEL_FIELDS.jump), j.tank_t);
    out.push({ tank: "jump", fuel: j.fuel || "jump fuel", full_t: j.tank_t, now_t: now, share: now / j.tank_t });
  }
  return out;
}

/** The ship as it is now, for planning: its own record with the tanks at the kept levels. */
export function fuelNow<S extends FuelShip>(ship: S, kept: (field: string) => unknown): S {
  const g = fuelGauges(ship, kept);
  const prop = g.find((x) => x.tank === "propellant");
  const jump = g.find((x) => x.tank === "jump");
  return { ...ship, ...(prop ? { propellant_t: prop.now_t } : {}), ...(jump && ship.jump ? { jump: { ...ship.jump, fuel_t: jump.now_t } } : {}) };
}

/** Tonnes in words, to a sensible precision. */
export function tonnesWords(t: number): string {
  return t >= 100 ? `${Math.round(t)} t` : t >= 10 ? `${t.toFixed(1)} t` : `${t.toFixed(2)} t`;
}

/** A ship's full numbers from the kernel: a stock id or a pack ship resolved (ship_at_world with no worlds). */
export async function shipRecordOf(session: { shipAtWorld(a: import("./ship-at-world.ts").ShipAtWorldAsk): Promise<{ ok: true; report: { ship: unknown } } | { ok: false; message: string }> }, nav: string | import("./route.ts").NavShip): Promise<{ ok: true; ship: Record<string, unknown> } | { ok: false; message: string }> {
  const r = await session.shipAtWorld({ fixture: "sol", ship: nav, worlds: [] });
  return r.ok ? { ok: true, ship: r.report.ship as Record<string, unknown> } : r;
}
