// Seats and audiences (V10, H8.7). One shared table by default; more seats
// only when the GM asks for them. An audience is a knowledge group: what it
// has been shown lives in the chronicle as grants per audience (K9), so two
// empires at one table can know different things about the same system.
// A seat is a place at the table: whose knowledge it shows, which screens it
// opens, what it may ask for and what applies without asking.
import { embedOrigin } from "./embed.ts";
import { checkBrand, type SeatBrand } from "./brand.ts";
import type { Intent, IntentKind } from "./intent.ts";
import { INTENT_KINDS } from "./intent.ts";
import { sharedTableSeat, type Seat } from "./seat.ts";
import type { ObserveSetup } from "./data-views.ts";

export const EVERYONE = "everyone";

export interface Audience {
  id: string;
  label: string;
  /** The tag that marks what this audience has discovered ("discovered:vesh"): fog of war by tag. */
  tag?: string;
  /** Data view in force before anything is granted to this audience (a fog-of-war audience starts with one that shows nothing). Absent: the campaign default. */
  starts_with?: string;
  /** Data view a discovery grants. Absent: the campaign's "before" view. */
  discovered_view?: string;
}

export interface SeatPlan {
  schema: "sse.seats/0";
  audiences: Audience[];
  seats: Seat[];
}

/** The default: everyone at one table, which may ask the GM to set a value. No seat UI is shown for it. */
export function defaultPlan(): SeatPlan {
  return { schema: "sse.seats/0", audiences: [{ id: EVERYONE, label: "Everyone" }], seats: [sharedTableSeat()] };
}

/** True when the plan is the default one table (the GM sees one line, not a seat list). */
export function isDefaultPlan(p: SeatPlan): boolean {
  return p.seats.length === 1 && p.seats[0]!.id === "table" && p.audiences.length === 1;
}

/** A lower-case id from a label ("Empire: Vesh" -> "empire-vesh"). */
export function slug(label: string): string {
  return (
    label
      .toLowerCase()
      .normalize("NFKD")
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .slice(0, 32) || "seat"
  );
}

const RESERVED = new Set([EVERYONE, "gm", "all"]);

/** Kernel grant field for an audience at an address (K9: `observe.view`, or `observe.view.<audience>`). */
export function grantField(audience: string): string {
  return audience === EVERYONE ? "observe.view" : `observe.view.${audience}`;
}

/**
 * Add a seat with its own audience. `fog` gives the audience a view that
 * shows nothing until the GM marks discoveries (fog of war). `names` lets the
 * seat ask for names; `sealed` serves it finished scenes only, never the seed,
 * from its own room; `autoNames` applies those names without asking, inside
 * `scope`.
 */
export function addSeat(p: SeatPlan, o: { label: string; fog?: { starts_with: string; discovered_view: string }; names?: boolean; autoNames?: string[]; screens?: string[]; sealed?: boolean; embedOrigins?: string[]; brand?: SeatBrand }): SeatPlan {
  let id = slug(o.label);
  const taken = new Set([...p.seats.map((s) => s.id), ...p.audiences.map((a) => a.id), ...RESERVED]);
  for (let n = 2; taken.has(id); n++) id = `${slug(o.label)}-${n}`;
  const audience: Audience = o.fog ? { id, label: o.label, tag: `discovered:${id}`, starts_with: o.fog.starts_with, discovered_view: o.fog.discovered_view } : { id, label: o.label };
  const seat: Seat = {
    id,
    label: o.label,
    audience: id,
    authority: "player",
    may_send: o.names ? ["set_value"] : [],
    ...(o.names && o.autoNames?.length ? { auto_approve: [{ intent: "set_value" as const, scope: [...o.autoNames], fields: ["name"] }] } : {}),
    sealed: o.sealed === true,
    ...(o.screens?.length ? { screens: [...o.screens] } : {}),
    ...(o.embedOrigins?.length ? { embed_origins: [...o.embedOrigins] } : {}),
    ...(o.brand ? { brand: { ...o.brand } } : {}),
  };
  return { ...p, audiences: [...p.audiences, audience], seats: [...p.seats, seat] };
}

/** Remove a seat, and its audience when no other seat uses it. */
export function removeSeat(p: SeatPlan, id: string): SeatPlan {
  const seat = p.seats.find((s) => s.id === id);
  if (!seat || id === "table") return p;
  const seats = p.seats.filter((s) => s.id !== id);
  const audiences = p.audiences.filter((a) => a.id === EVERYONE || a.id !== seat.audience || seats.some((s) => s.audience === a.id));
  return { ...p, audiences, seats };
}

/** Why a plan cannot be used, one line per problem; empty when it is sound. */
export function checkSeats(p: SeatPlan, ctx: { views?: string[]; screens?: string[] } = {}): string[] {
  const out: string[] = [];
  if (p.schema !== "sse.seats/0") out.push("not a seats document (sse.seats/0)");
  const aud = new Set<string>();
  for (const a of p.audiences) {
    if (!/^[a-z0-9][a-z0-9-]*$/.test(a.id)) out.push(`audience id ${a.id} must be lower-case letters, digits and dashes`);
    if (aud.has(a.id)) out.push(`two audiences are called ${a.id}`);
    if (a.id !== EVERYONE && RESERVED.has(a.id)) out.push(`audience id ${a.id} is reserved`);
    aud.add(a.id);
    for (const v of [a.starts_with, a.discovered_view]) if (v && ctx.views && !ctx.views.includes(v)) out.push(`audience ${a.label}: no data view ${v}`);
  }
  if (!aud.has(EVERYONE)) out.push("the audience everyone always exists");
  const ids = new Set<string>();
  for (const s of p.seats) {
    if (ids.has(s.id)) out.push(`two seats are called ${s.id}`);
    ids.add(s.id);
    if (s.authority !== "player") out.push(`seat ${s.label}: only the GM's own seat has GM authority`);
    if (!aud.has(s.audience)) out.push(`seat ${s.label}: no audience ${s.audience}`);
    if (s.lens && ctx.views && !ctx.views.includes(s.lens)) out.push(`seat ${s.label}: no data view ${s.lens}`);
    for (const k of s.may_send) if (!INTENT_KINDS.includes(k as IntentKind)) out.push(`seat ${s.label}: unknown intent ${k}`);
    for (const a of s.auto_approve ?? []) if (!s.may_send.includes(a.intent)) out.push(`seat ${s.label}: applies ${a.intent} without asking but may not send it`);
    for (const sc of s.screens ?? []) if (ctx.screens && !ctx.screens.includes(sc)) out.push(`seat ${s.label}: no screen ${sc}`);
    if (s.sealed && s.id === "table") out.push("the table cannot be sealed: it is the open link");
    if (s.brand) for (const b of checkBrand(s.brand)) out.push(`seat ${s.label}: ${b}`);
    for (const o of s.embed_origins ?? []) {
      const r = embedOrigin(o);
      if (!r.ok) out.push(`seat ${s.label}: tabletop ${r.reason}`);
      else if (r.origin !== o) out.push(`seat ${s.label}: tabletop ${o} should be written ${r.origin}`);
    }
  }
  if (!ids.has("table")) out.push("the table seat always exists");
  return out;
}

/**
 * The intents that set each fog-of-war audience's starting view on the
 * universe root (K9 reads `observe.view.<audience>.default` there). The GM
 * sends them once when the seat is added.
 */
export function startIntents(p: SeatPlan, root: string, at_yr: number): Intent[] {
  return p.audiences
    .filter((a) => a.id !== EVERYONE && a.starts_with)
    .map((a) => ({ kind: "grant" as const, address: root, audience: a.id, view: a.starts_with!, default: true, at_yr }));
}

/**
 * Mark `address` (and everything in it: moons follow their planet) as
 * discovered by an audience, or take the discovery back. A grant per
 * audience, chronicled, so "when did Vesh learn of Tarn" is history.
 */
export function discoverIntent(a: Audience, address: string, known: boolean, fallbackView: string, at_yr: number): Intent {
  // Taking a discovery back removes the grant, so the body falls back to its parent, its tags and the audience's starting view.
  return { kind: "grant", address, audience: a.id, view: known ? (a.discovered_view ?? fallbackView) : null, at_yr };
}

/** The join link for a seat ("table" is the plain room link) and, optionally, one of its screens. */
export function joinLink(base: string, room: string, seat = "table", screen?: string): string {
  const q = new URLSearchParams({ join: room });
  if (seat !== "table") q.set("seat", seat);
  if (screen) q.set("screen", screen);
  return new URL(`players.html?${q}`, base).href;
}

/**
 * The views fog of war needs from the campaign's data views: one that shows
 * nothing at all (the audience does not learn a body exists) to start with,
 * and the campaign's "before" view for a discovery. Null when the campaign
 * has no view that shows nothing.
 */
export function fogViews(setup: Pick<ObserveSetup, "views" | "campaign"> | null | undefined): { starts_with: string; discovered_view: string } | null {
  if (!setup) return null;
  const blank = setup.views.find((v) => v.show_tiers.length === 0 && !v.overrides?.some((o) => o.show) && v.tags.mode === "none" && v.unknown_style === "omit");
  return blank ? { starts_with: blank.id, discovered_view: setup.campaign.before } : null;
}

/** The join link for a sealed seat: its own room, never the shared one (whose events carry every fact). */
export function sealedLink(base: string, room: string, seat: string, screen?: string): string {
  const q = new URLSearchParams({ sealed: room, seat });
  if (screen) q.set("screen", screen);
  return new URL(`players.html?${q}`, base).href;
}
