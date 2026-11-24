// What is in a system, in one line (V3 gap 39, "hover a system to see what is in it"). V3's starmap told the GM, on
// hover, how many stars, planets, moons, minor bodies and ships a system held, any life, and anything remarkable,
// without opening it. Here it is one pure reader over whatever the caller already holds for a system: its scene
// subjects (kind, tags, stamps) or the picker's items (kind chips only). Nothing is asked of the kernel, so a hover
// costs nothing ("no heavy work for a click"). It is a GM-only count: it names honesty marks and hidden bodies, so a
// players' screen never shows it.
import { subjectKinds } from "./picker.ts";

/** The least a body needs for the count: its kind, or failing that the picker's kind chips. */
export interface SummaryBody {
  label?: string;
  /** Scene kind (star, planet, dwarf_planet, moon, belt, ring, station, ...). */
  kind?: string;
  /** Picker chips ("Stars", "Planets", ...), used when there is no kind. */
  kinds?: readonly string[];
  tags?: readonly string[];
  /** Honesty and evidence stamps (authored, impossible, fictional, ...). */
  stamps?: readonly string[];
  /** A ring round a world (a belt whose host is a planet). */
  ring?: unknown;
  parent?: string | null;
}

export interface SystemSummary {
  stars: number;
  planets: number;
  /** Dwarf planets, asteroids and comets named one by one. */
  minor: number;
  moons: number;
  /** Belts, discs and clouds (populations, counted once each); rings are counted under `rings`. */
  belts: number;
  rings: number;
  /** Ships, stations, bases and other constructs. */
  craft: number;
  /** Names of bodies the data marks as living (a life or biosphere tag, or a Life chip). */
  life: string[];
  /** Short GM-only notes: a multiple star, a remnant, honesty marks, ringed worlds. */
  remarkable: string[];
}

const LIFE_TAG = /^(life|biosphere|inhabited)(\b|[:/])/i;
const lives = (b: SummaryBody) => (b.tags ?? []).some((t) => LIFE_TAG.test(t)) || (b.kinds ?? []).some((k) => /^life$/i.test(k));
const n = (count: number, one: string, many = `${one}s`) => `${count.toLocaleString("en-GB")} ${count === 1 ? one : many}`;

/** Count a system's bodies and note what stands out. Unknown kinds are left out of the counts, never guessed. */
export function systemSummary(bodies: readonly SummaryBody[]): SystemSummary {
  const s: SystemSummary = { stars: 0, planets: 0, minor: 0, moons: 0, belts: 0, rings: 0, craft: 0, life: [], remarkable: [] };
  let remnants = 0;
  let browns = 0;
  let impossible = 0;
  let fictional = 0;
  const ringed = new Set<string>();
  for (const b of bodies) {
    const k = b.kind;
    const chips = k ? subjectKinds({ kind: k }) : (b.kinds ?? []);
    if (k === "remnant") remnants++;
    if (k === "brown_dwarf") browns++;
    if (k === "dwarf_planet" || k === "asteroid" || k === "comet") s.minor++;
    else if (k === "ring" || (b.ring && (k === "belt" || !k))) {
      s.rings++;
      if (b.parent) ringed.add(b.parent);
    } else if (chips.includes("Stars")) s.stars++;
    else if (chips.includes("Planets")) s.planets++;
    else if (chips.includes("Moons")) s.moons++;
    else if (chips.includes("Belts and rings")) s.belts++;
    else if (chips.includes("Ships and stations")) s.craft++;
    if (lives(b)) s.life.push(b.label ?? "a world");
    if (b.stamps?.includes("impossible")) impossible++;
    if (b.stamps?.includes("fictional")) fictional++;
  }
  if (s.stars === 2) s.remarkable.push("a binary star");
  else if (s.stars > 2) s.remarkable.push(`a ${s.stars}-star system`);
  if (remnants) s.remarkable.push(remnants === 1 ? "a dead star (a remnant)" : n(remnants, "dead star"));
  if (browns) s.remarkable.push(n(browns, "brown dwarf"));
  if (ringed.size) s.remarkable.push(n(ringed.size, "ringed world"));
  if (impossible) s.remarkable.push(`${n(impossible, "body", "bodies")} marked impossible`);
  if (fictional) s.remarkable.push(`${n(fictional, "body", "bodies")} marked fictional`);
  return s;
}

/** The summary as one short line for a hover or a picker row: "1 star · 8 planets · 290 moons · life on Earth". */
export function summaryWords(s: SystemSummary): string {
  const parts = [
    s.stars ? n(s.stars, "star") : "",
    s.planets ? n(s.planets, "planet") : "",
    s.moons ? n(s.moons, "moon") : "",
    s.minor ? n(s.minor, "minor body", "minor bodies") : "",
    s.belts ? n(s.belts, "belt") : "",
    s.craft ? n(s.craft, "ship or station", "ships and stations") : "",
    s.life.length ? `life on ${s.life.length <= 2 ? s.life.join(" and ") : n(s.life.length, "world")}` : "",
  ].filter(Boolean);
  return parts.length ? parts.join(" · ") : "Nothing charted yet";
}
