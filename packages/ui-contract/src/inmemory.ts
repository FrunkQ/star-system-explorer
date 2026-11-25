// In memory (track SH6, design `handover/design/SSE4-spaceflight-history-design.md` section 8): one still page of the
// names of those who died flying to space, preparing to, or working to send others, from a pack's `in-memory.json`
// (RS9 part 2) and its people.
//
// Rules of restraint (design 8.1, standing; Alex: "one place we don't joke"): every name and date comes from the pack
// with its sources; nothing here invents, ranks or decorates. Names are shown as the record gives them; an event's
// sentence is the pack's own. Where the sources give more dead than names, the page says so in the pack's words.
import type { Json } from "./json.ts";
import { shownName } from "./rollcall.ts";

export interface MemorySource {
  text: string;
  url?: string;
}

export interface MemoryName {
  name: string;
  /** The pack person, when they are one (a link to their flights). */
  person?: string;
  being?: string;
}

export interface MemoryLoss {
  id: string;
  at_s: number;
  /** The date the record itself uses, "1986-01-28". */
  date: string;
  where: string;
  during: string;
  sentence: string;
  /** The craft's flight log, when it has one. */
  on: string | null;
  names: MemoryName[];
  unconfirmed: string | null;
  /** Sourced number of dead, or [low, high] when the sources differ. */
  toll: number | [number, number] | null;
  toll_why?: string;
  note?: string;
  sources: MemorySource[];
}

export interface MemoryPart {
  /** "flight", "preparing", "ground", "animals". */
  key: string;
  heading: string;
  losses: MemoryLoss[];
}

export interface InMemory {
  parts: MemoryPart[];
  /** People (not animals) across every loss: low and high ends when a toll is disputed. */
  people: [number, number];
  /** Memorials on Earth, as text with sources (ruling 10). */
  earth: { name: string; where?: string; sources: MemorySource[] }[];
}

const PARTS: { key: string; heading: string; during: readonly string[] }[] = [
  { key: "flight", heading: "On a flight", during: ["flight"] },
  { key: "preparing", heading: "Preparing to fly", during: ["training", "ground_test"] },
  { key: "ground", heading: "Working to send others", during: ["ground_work"] },
];

const str = (v: Json | undefined): string | undefined => (typeof v === "string" && v.trim() ? v : undefined);
const sourcesOf = (v: Json | undefined): MemorySource[] =>
  (Array.isArray(v) ? v : []).flatMap((s) => {
    const o = s as Record<string, Json>;
    const text = str(o?.text);
    return text ? [{ text, ...(str(o.url) ? { url: str(o.url)! } : {}) }] : [];
  });

/** The page from a pack's `in-memory.json` and its `crews.json` (for names and kinds of being of pack people). */
export function inMemory(doc: Json, crews: Json | null): InMemory {
  const people = new Map<string, { name: string; being?: string }>();
  for (const p of ((crews as { people?: Json[] } | null)?.people ?? []) as Record<string, Json>[]) {
    people.set(String(p.id), { name: shownName(String(p.name)), ...(str(p.being) ? { being: str(p.being)! } : {}) });
  }
  const d = doc as { losses?: Json[]; earth_memorials?: Json[] };
  const losses: MemoryLoss[] = (d.losses ?? []).map((l) => {
    const o = l as Record<string, Json>;
    const names: MemoryName[] = [
      ...((Array.isArray(o.people) ? o.people : []) as string[]).map((id) => {
        const p = people.get(id);
        return { name: p?.name ?? id, person: id, ...(p?.being ? { being: p.being } : {}) };
      }),
      ...((Array.isArray(o.names) ? o.names : []) as Record<string, Json>[]).flatMap((n) => (str(n.name) ? [{ name: shownName(str(n.name)!) }] : [])),
    ];
    const toll = o.toll;
    return {
      id: String(o.id),
      at_s: Number(o.at_s),
      date: String(o.date ?? ""),
      where: String(o.where ?? ""),
      during: String(o.during ?? ""),
      sentence: String(o.sentence ?? ""),
      on: str(o.on) ?? null,
      names,
      unconfirmed: str(o.unconfirmed) ?? null,
      toll: typeof toll === "number" ? toll : Array.isArray(toll) && toll.length === 2 ? [Number(toll[0]), Number(toll[1])] : null,
      ...(str(o.toll_why) ? { toll_why: str(o.toll_why)! } : {}),
      ...(str(o.note) ? { note: str(o.note)! } : {}),
      sources: sourcesOf(o.sources),
    };
  });
  losses.sort((a, b) => a.at_s - b.at_s || (a.id < b.id ? -1 : 1));
  const isAnimal = (n: MemoryName) => !!n.being && n.being !== "human";
  const parts: MemoryPart[] = [];
  for (const p of PARTS) {
    const mine = losses
      .filter((l) => p.during.includes(l.during))
      .map((l) => ({ ...l, names: l.names.filter((n) => !isAnimal(n)) }))
      .filter((l) => l.names.length || l.unconfirmed);
    if (mine.length) parts.push({ key: p.key, heading: p.heading, losses: mine });
  }
  // Animals that died on flights: a short separate part at the foot, facts only (ruling 6).
  const animals = losses.filter((l) => l.names.some(isAnimal)).map((l) => ({ ...l, names: l.names.filter(isAnimal), unconfirmed: null }));
  if (animals.length) parts.push({ key: "animals", heading: "Animals that died on flights", losses: animals });
  let lo = 0;
  let hi = 0;
  for (const l of losses) {
    const humans = l.names.filter((n) => !isAnimal(n)).length;
    const [a, b] = l.toll === null ? [humans, humans] : typeof l.toll === "number" ? [l.toll, l.toll] : l.toll;
    // A toll counts every death in the event; where it is smaller than the names given, the names stand.
    lo += Math.max(a, humans);
    hi += Math.max(b, humans);
  }
  const earth = ((d.earth_memorials ?? []) as Record<string, Json>[]).flatMap((e) => (str(e.name) ? [{ name: str(e.name)!, ...(str(e.where) ? { where: str(e.where)! } : {}), sources: sourcesOf(e.sources) }] : []));
  return { parts, people: [lo, hi], earth };
}

/** The page's first line, computed: "N people have died flying to space, preparing to, or working to send others." */
export function inMemoryLine(m: InMemory, say: (n: number) => string = String): string {
  const [lo, hi] = m.people;
  const n = lo === hi ? `${say(lo)} people have` : `Between ${say(lo)} and ${say(hi)} people have`;
  return `${n} died flying to space, preparing to, or working to send others. These are their names.`;
}

/** A record's ISO date ("1986-01-28") written with a calendar's month names: "28 January 1986". The text when it is not one. */
export function recordDate(date: string, months?: readonly { name: string }[]): string {
  const m = /^(-?\d{1,6})-(\d{2})-(\d{2})$/.exec(date);
  if (!m || !months?.length) return date;
  const month = months[Number(m[2]) - 1]?.name;
  return month ? `${Number(m[3])} ${month} ${Number(m[1])}` : date;
}

/** A prepared correction: a new-issue address with the entry named. Opening it sends nothing (design 8.1 rule 7). */
export function correctionUrl(loss: MemoryLoss | null, repo = "FrunkQ/star-system-explorer"): string {
  const title = loss ? `In memory: ${loss.date}, ${loss.where}` : "In memory: something wrong or missing";
  const body = [
    loss ? `Entry: ${loss.id} (${loss.date}, ${loss.where})` : "Entry: (new, or which one)",
    "",
    "What is wrong or missing:",
    "",
    "Two independent sources (title and address):",
    "1. ",
    "2. ",
    "",
    "_Sent from the In memory page of Star System Explorer. A correction is a pack data change with its sources._",
  ].join("\n");
  const q = new URLSearchParams({ title, body });
  return `https://github.com/${repo}/issues/new?${q.toString()}`;
}

/** The losses on the day [from_s, to_s), once each, for the day of a loss (design 8.4). */
export function lossesOn(m: InMemory, from_s: number, to_s: number): MemoryLoss[] {
  const seen = new Set<string>();
  return m.parts.flatMap((p) => p.losses).filter((l) => l.at_s >= from_s && l.at_s < to_s && !seen.has(l.id) && (seen.add(l.id), true));
}
