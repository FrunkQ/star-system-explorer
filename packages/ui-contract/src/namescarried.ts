// Names carried (track SH7, design `handover/design/SSE4-spaceflight-history-design.md` section 8.5): the real
// memorials in space from a pack's `memorials.json` (RS9 part 3), shown with the names of those they are for.
//
// Rules of restraint (design 8.1, standing): every memorial is the pack's, with its sources; nothing is invented (no
// monument, question 5). A memorial takes its name only from the date it was given, when the pack knows that date;
// when it does not, the pack's reason is said instead of a guess. Nothing here feeds physics or the state hash.
//
// The small bodies are on their real orbits: `orbitAt` places one from the pack's osculating elements (JPL Horizons)
// by a two-body Kepler solve about the Sun, for the chart's layer and the Sky view's "tonight" (fudge register,
// "Names carried").
import type { Json } from "./json.ts";
import { shownName } from "./rollcall.ts";
import { constant } from "./sheet.ts";
import type { MemorySource } from "./inmemory.ts";

export interface OrbitElements {
  a_m: number;
  e: number;
  i_deg: number;
  node_deg: number;
  peri_deg: number;
  m0_deg: number;
  epoch_s: number;
  frame: string;
}

export interface CarriedName {
  id: string;
  /** "named_small_body", and later "object_left", "named_feature", "remains". */
  kind: string;
  /** The memorial's own name, "3352 McAuliffe". */
  name: string;
  /** Who it is for, as the pack's people give them. */
  for: { name: string; person?: string }[];
  /** When it took its name, or null with `since_why`. */
  since_s: number | null;
  since_date: string | null;
  since_why: string | null;
  elements: OrbitElements | null;
  sources: MemorySource[];
}

const str = (v: Json | undefined): string | undefined => (typeof v === "string" && v.trim() ? v : undefined);
const num = (v: Json | undefined): number | null => (typeof v === "number" && Number.isFinite(v) ? v : null);

function elementsOf(v: Json | undefined): OrbitElements | null {
  const o = (v ?? null) as Record<string, Json> | null;
  if (!o || typeof o !== "object") return null;
  const keys = ["a_m", "e", "i_deg", "node_deg", "peri_deg", "m0_deg", "epoch_s"] as const;
  const vals = keys.map((k) => num(o[k]));
  if (vals.some((x) => x === null)) return null;
  const [a_m, e, i_deg, node_deg, peri_deg, m0_deg, epoch_s] = vals as number[];
  // Only closed orbits are placed here.
  if (!(a_m! > 0) || !(e! >= 0 && e! < 1)) return null;
  return { a_m: a_m!, e: e!, i_deg: i_deg!, node_deg: node_deg!, peri_deg: peri_deg!, m0_deg: m0_deg!, epoch_s: epoch_s!, frame: str(o.frame) ?? "" };
}

/** The memorials from a pack's `memorials.json` and its `crews.json` (for the names of pack people), in name order. */
export function namesCarried(doc: Json, crews: Json | null): CarriedName[] {
  const people = new Map<string, string>();
  for (const p of ((crews as { people?: Json[] } | null)?.people ?? []) as Record<string, Json>[]) people.set(String(p.id), shownName(String(p.name)));
  const list = ((doc as { memorials?: Json[] } | null)?.memorials ?? []) as Record<string, Json>[];
  const out = list.flatMap((m): CarriedName[] => {
    const name = str(m.name);
    if (!name) return [];
    const ids = (Array.isArray(m.people) ? m.people : []).filter((x): x is string => typeof x === "string");
    const forList = ids.length ? ids.map((id) => ({ name: people.get(id) ?? id, person: id })) : str(m.names_text) ? [{ name: shownName(str(m.names_text)!) }] : [];
    const sources = (Array.isArray(m.sources) ? m.sources : []).flatMap((s) => {
      const o = s as Record<string, Json>;
      const text = str(o?.text);
      return text ? [{ text, ...(str(o.url) ? { url: str(o.url)! } : {}) }] : [];
    });
    const where = (m.where ?? null) as Record<string, Json> | null;
    return [
      {
        id: String(m.id ?? name),
        kind: str(m.kind) ?? "",
        name,
        for: forList,
        since_s: num(m.since_s),
        since_date: str(m.since_date) ?? null,
        since_why: str(m.since_why) ?? null,
        elements: elementsOf(where?.elements),
        sources,
      },
    ];
  });
  // Minor-planet numbers first in number order, then by name: the order the record itself uses.
  const key = (c: CarriedName) => Number(/^(\d+)\s/.exec(c.name)?.[1] ?? Number.MAX_SAFE_INTEGER);
  return out.sort((a, b) => key(a) - key(b) || (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
}

/** Whether a memorial carries its name at `at_s`: from its date when the pack knows it; always when it does not. */
export const namedAt = (c: CarriedName, at_s: number): boolean => c.since_s === null || at_s >= c.since_s;

/** How a memorial's naming date is said: the date in words, or the pack's reason it is not known. */
export function sinceWords(c: CarriedName, dateText: (iso: string) => string): string {
  if (c.since_date) return `Named ${dateText(c.since_date)}.`;
  return c.since_why ? `The naming date is not given here: ${c.since_why}.` : "The naming date is not given here.";
}

/** When no memorial has a naming date and they all give the same reason, that reason, to be said once for the list. */
export function sharedSinceWhy(list: readonly CarriedName[]): string | null {
  if (!list.length || list.some((c) => c.since_date || !c.since_why)) return null;
  const why = list[0]!.since_why!;
  return list.every((c) => c.since_why === why) ? why : null;
}

/** One line of the In memory page's "Names carried" part: "3352 McAuliffe, for Sharon Christa McAuliffe." */
export function carriedLine(c: CarriedName): string {
  const who = c.for.map((f) => f.name);
  const list = who.length <= 1 ? (who[0] ?? "") : `${who.slice(0, -1).join(", ")} and ${who[who.length - 1]}`;
  return list ? `${c.name}, for ${list}.` : `${c.name}.`;
}

export interface OrbitPlace {
  /** Position about the Sun in the elements' frame (heliocentric ecliptic J2000), metres. */
  x: number;
  y: number;
  z: number;
  /** Distance from the Sun, metres. */
  r_m: number;
}

const RAD = Math.PI / 180;

/** Where a small body is at `at_s`, by a two-body Kepler solve about the Sun from its osculating elements. */
export function orbitAt(el: OrbitElements, at_s: number): OrbitPlace {
  const n = Math.sqrt(constant("GM_sun") / (el.a_m * el.a_m * el.a_m));
  const tau = 2 * Math.PI;
  let M = (el.m0_deg * RAD + n * (at_s - el.epoch_s)) % tau;
  if (M < 0) M += tau;
  // Newton's method on E - e sin E = M; from E = pi for high e, which always converges.
  let E = el.e < 0.8 ? M : Math.PI;
  for (let k = 0; k < 50; k++) {
    const d = (E - el.e * Math.sin(E) - M) / (1 - el.e * Math.cos(E));
    E -= d;
    if (Math.abs(d) < 1e-13) break;
  }
  const px = el.a_m * (Math.cos(E) - el.e);
  const py = el.a_m * Math.sqrt(1 - el.e * el.e) * Math.sin(E);
  const [O, i, w] = [el.node_deg * RAD, el.i_deg * RAD, el.peri_deg * RAD];
  const [cO, sO, ci, si, cw, sw] = [Math.cos(O), Math.sin(O), Math.cos(i), Math.sin(i), Math.cos(w), Math.sin(w)];
  const x = (cO * cw - sO * sw * ci) * px + (-cO * sw - sO * cw * ci) * py;
  const y = (sO * cw + cO * sw * ci) * px + (-sO * sw + cO * cw * ci) * py;
  const z = sw * si * px + cw * si * py;
  return { x, y, z, r_m: Math.hypot(x, y, z) };
}
