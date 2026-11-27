// Where Roll call's people and logs come from (SH2). One `RollSource` whichever way:
// - a pack in the campaign's stack that holds a roll (crews, a calendar and flight logs), such as "Sol today" once it
//   is added through the Open door like any pack (CM0): its files are read from the pack itself (`heldFiles`);
// - a "Sol today" folder (crews.json, pack.json, logs/) named by the address bar's `?spaceflight=<folder URL>`, kept
//   as a shortcut (SH8 part 1): the same files, fetched from the folder;
// - otherwise the open campaign: its people (manifest entries) and its constructs' flight logs, read from the kernel.
// The logs of the day's craft are read as they are needed; a pack's `logs/index.json` says where each one is (RS9
// part 2); without it the feed looks in the pack's three log folders and cannot follow a docking by catalogue id.
import { GREGORIAN, heldFiles, PERSON, inMemory, namesCarried, type CarriedName, type InMemory, entryFrom, indexRoll, logFrom, sourceFromCrews, sourceFromManifest, drawnSecond, type Calendar, type Json, type RecordRead, type RollIndex, type RollLog, type ViewContext } from "../../../ui-contract/src/index.ts";

export interface RollFeed {
  ix: RollIndex;
  calendar: Calendar;
  /** Where the people and logs came from, in words, with the credit the source asks for. */
  from: string;
  credit?: string;
  /** Fetch the logs these constructs need (and what they dock to); resolves with the index rebuilt when any arrived. */
  need(ids: readonly string[]): Promise<RollIndex | null>;
  /** In memory (SH6): the pack's names of those who died, when it gives them. */
  memory?: InMemory;
  /** Names carried (SH7): the pack's real memorials in space, when it gives them. */
  carried?: CarriedName[];
  /** The pack's own count steps, when it ships them. */
  steps?: [number, number][];
  /** True when every log is at hand, so "also in flight" can be counted honestly. */
  allLogs: boolean;
  /** The pack's span, for opening on a sensible day. */
  span?: { from_s: number; to_s: number };
}

/** The `?spaceflight=` folder, or null. */
export function packFolder(): string | null {
  if (typeof location === "undefined") return null;
  const raw = new URLSearchParams(location.search).get("spaceflight");
  if (!raw) return null;
  try {
    const u = new URL(raw, document.baseURI);
    return u.href.endsWith("/") ? u.href : `${u.href}/`;
  } catch {
    return null;
  }
}

async function getJson(url: string): Promise<Json | null> {
  try {
    const r = await fetch(url);
    return r.ok ? ((await r.json()) as Json) : null;
  } catch {
    return null;
  }
}

/** Reads one file of a pack by its path in the pack's folder: JSON, or null when it is not there. */
export type PackRead = (path: string) => Promise<Json | null>;

/** A pack folder on a server, read file by file. */
export const folderRead = (base: string): PackRead => (p) => getJson(`${base}${p}`);

/** The top pack in the stack that holds a roll (its crews file among its held files), as a reader and its name. */
export function heldRoll(packs: readonly unknown[]): { read: PackRead; label: string } | null {
  for (const p of [...packs].reverse()) {
    const held = heldFiles(p);
    const other = (p as { provides?: { other?: { crews?: unknown; about?: { label?: unknown } } } }).provides?.other;
    if (!held || typeof other?.crews !== "string" || !held(other.crews)) continue;
    const label = typeof other.about?.label === "string" ? other.about.label : String((p as { id?: unknown }).id ?? "the pack");
    // The pack document itself is pack.json, as it is in the folder.
    return { read: async (path) => (path === "pack.json" ? (p as Json) : held(path)), label };
  }
  return null;
}

/** Open a pack's roll, from its folder or from the stack. Null with the reason when it cannot be read. */
export async function openPack(read: PackRead, label = "Sol today"): Promise<RollFeed | { why: string }> {
  const [pack, index] = await Promise.all([read("pack.json"), read("logs/index.json")]);
  const other = (pack as { provides?: { other?: { crews?: unknown; in_memory?: unknown; memorials?: unknown; calendar?: Calendar } } } | null)?.provides?.other;
  const fileOf = (f: unknown) => (typeof f === "string" && !f.includes("..") ? f : null);
  const crewsFile = fileOf(other?.crews) ?? "crews.json";
  const memFile = fileOf(other?.in_memory);
  const carFile = fileOf(other?.memorials);
  const [crews, memDoc, carDoc] = await Promise.all([read(crewsFile), memFile ? read(memFile) : null, carFile ? read(carFile) : null]);
  if (!crews) return { why: `No crews could be read from ${label}.` };
  const cal = other?.calendar;
  if (!cal) return { why: "This copy of the pack gives no calendar, so its days cannot be written. A newer copy of Sol today has one." };
  // logs/index.json: { about, logs: [{ construct, path (under logs/), name, gcat, from_s, to_s }] }.
  const listed = (index as { logs?: unknown } | null)?.logs;
  const rows = Array.isArray(listed) ? (listed as { construct: string; path: string; gcat?: string }[]) : null;
  const where = new Map<string, string>();
  for (const r of rows ?? []) {
    where.set(r.construct, r.path);
    if (r.gcat) where.set(r.gcat.startsWith("gcat:") ? r.gcat : `gcat:${r.gcat}`, r.path);
  }
  const logs = new Map<string, RollLog>();
  const missing = new Set<string>();
  const source = sourceFromCrews(crews, []);
  const steps = (crews as { in_space?: [number, number][] }).in_space;
  const fetchLog = async (id: string): Promise<RollLog | null> => {
    if (logs.has(id)) return logs.get(id)!;
    if (missing.has(id)) return null;
    const paths = where.has(id) ? [where.get(id)!] : id.startsWith("gcat:") ? [] : [`${id}.json`, `earth-orbit/${id}.json`, `departed/${id}.json`];
    for (const p of paths) {
      const doc = await read(`logs/${p}`);
      if (doc && typeof doc === "object" && !Array.isArray(doc) && "pieces" in doc) {
        const l = logFrom(doc);
        logs.set(id, l);
        logs.set(l.construct, l);
        if (l.gcat) logs.set(l.gcat.startsWith("gcat:") ? l.gcat : `gcat:${l.gcat}`, l);
        return l;
      }
    }
    missing.add(id);
    return null;
  };
  const rebuild = () => indexRoll({ ...source, logs: [...new Set(logs.values())] });
  let ix = rebuild();
  const starts = source.stays.map((s) => s.from_s);
  const ends = source.stays.map((s) => s.to_s ?? s.from_s);
  return {
    get ix() {
      return ix;
    },
    calendar: cal,
    from: label,
    allLogs: false,
    ...(source.credit ? { credit: source.credit } : {}),
    ...(steps ? { steps } : {}),
    ...(memDoc && crews ? { memory: inMemory(memDoc, crews) } : {}),
    ...(carDoc ? { carried: namesCarried(carDoc, crews) } : {}),
    span: { from_s: Math.min(...starts), to_s: Math.max(...ends) },
    async need(ids) {
      const before = logs.size;
      const got = await Promise.all(ids.filter((id) => !logs.has(id) && !missing.has(id)).map(fetchLog));
      // What they dock to: one step further.
      const hosts = got.flatMap((l) => (l ? l.pieces.flatMap((p) => (p.state.kind === "docked" && p.state.to && !logs.has(p.state.to) ? [p.state.to] : [])) : []));
      await Promise.all([...new Set(hosts)].map(fetchLog));
      if (logs.size === before) return null;
      ix = rebuild();
      return ix;
    },
  };
}

/** The open campaign's people and the flight logs of whatever they were aboard. */
export async function openCampaign(ctx: ViewContext, system: string): Promise<RollFeed> {
  const session = ctx.session as ViewContext["session"] & { record?: (a: string) => Promise<{ ok: true; record: RecordRead } | { ok: false }>; flightLog?: (ask: { construct: string }) => Promise<{ ok: true; log: Json } | { ok: false }> };
  const ws = ctx.workspace;
  const root = system.split("/")[0] ?? system;
  const entries = [];
  if (session.record) {
    const top = await session.record(root);
    if (top.ok) {
      const mine = top.record.actors.filter((a) => a.kind === PERSON);
      const reads = await Promise.all(mine.map((a) => session.record!(a.address)));
      for (let i = 0; i < mine.length; i++) {
        const r = reads[i]!;
        entries.push(entryFrom(mine[i]!, r.ok ? r.record : null));
      }
    }
  }
  const ons = [...new Set(entries.flatMap((e) => e.moves.map((m) => (typeof m.value === "string" ? m.value : "")).filter(Boolean)))];
  const logs: RollLog[] = [];
  if (session.flightLog) {
    const reads = await Promise.all(ons.map((c) => session.flightLog!({ construct: c })));
    reads.forEach((r, i) => {
      if (!r.ok) return;
      const l = logFrom({ construct: ons[i]!, ...(r.log as Record<string, Json>) });
      if (l.pieces.length) logs.push(l);
    });
  }
  const pc = ws.pack_calendar;
  const calendar: Calendar = pc
    ? { name: pc.name, epoch_s: pc.epoch_s, ...(pc.epoch_year !== undefined ? { epoch_year: pc.epoch_year } : {}), day_s: pc.day_s, year_days: pc.year_days, ...(pc.months?.length ? { months: pc.months } : {}), format: pc.format }
    : { name: "Days", epoch_s: Number(drawnSecond(ws.birth_s, ws.campaign_epoch_yr ?? 0)), day_s: GREGORIAN.day_s, year_days: 365, format: "Year {year}, day {day}" };
  const ix = indexRoll(sourceFromManifest(entries, logs, undefined));
  return { ix, calendar, from: "this campaign", allLogs: true, need: async () => null };
}
