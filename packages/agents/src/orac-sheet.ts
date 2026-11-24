// Orac and the master data sheet (A2 part 5, Alex 24 Nov): when an editing
// screen cannot find a gas or substance, Orac drafts it as data sheet rows
// (`provides.reference`, MP1) in the campaign's own pack. The check is the
// kernel's: the stack is tried with the rows, and the data sheet's load
// checks refuse an unknown element in a formula, a typed molar mass that
// disagrees with the formula, a vapour curve that misses its own triple
// point, a `src` naming no source, and an override that does not say what it
// replaces. An invented substance goes into a separate pack marked fictional,
// so everything that reads it carries the fictional mark.
import { stackPacks, type PackDoc, type ViewSession, type Workspace } from "../../ui-contract/src/index.ts";
import type { Entry, OracTarget } from "./orac.ts";
import handbook from "./orac-handbook.json" with { type: "json" };

export const REFERENCE_PACK = "campaign.reference";
export const FICTIONAL_REFERENCE_PACK = "campaign.reference.fictional";

type Row = { id: string; [k: string]: unknown };
export interface SubstanceDraft {
  substances: Row[];
  /** New source rows the values cite: `{id, text, doi?, url?, status}`. */
  sources?: Row[];
  /** Invented matter: the rows go into a pack marked fictional. */
  fictional?: boolean;
}

const obj = (x: unknown): x is Record<string, unknown> => !!x && typeof x === "object" && !Array.isArray(x);
const listOf = <T>(x: unknown): T[] => (Array.isArray(x) ? (x as T[]) : []);
const bump = (v: string) => {
  const [a = 0, b = 0, c = 0] = v.split(".").map(Number);
  return `${a}.${b}.${c + 1}`;
};
const packsOf = (session: ViewSession): PackDoc[] => {
  const s = session.stack();
  return [...s.base, ...(s.campaign ? [s.campaign] : [])];
};
const sheetOf = (p: PackDoc) => (obj(p.provides?.reference) ? (p.provides!.reference as Record<string, unknown>) : {});
const rowsOf = (session: ViewSession, table: string) => packsOf(session).flatMap((p) => listOf<Row>(sheetOf(p)[table]));
const nameOf = (r: Row) => {
  const n = r.names as { plain?: string; physics?: string } | undefined;
  return n?.plain || n?.physics || (r.name as string | undefined) || r.id;
};

export function substanceTarget(session: ViewSession, ws: Workspace): OracTarget<{ doc: SubstanceDraft; added: PackDoc[] }> {
  return {
    kind: "substance",
    what: "A substance for the master data sheet (a gas, liquid, ice or salt the editing screens could not find): its formula and the properties blocks read (phase points, vapour curve, density, optical values), every value with its unit and the source it comes from. Check first that it is not already there (need sheet:substances). Real matter cites published values; invented matter is marked fictional. Read a shipped row first (need substance:<id>) and the guide (need guide:data-sheet).",
    form: [
      '{"substances": [{"id": "<lower-case formula-like id>", "formula": {"<element id>": <count>}, "names": {"physics": "<name>", "plain": "<name>", "story": "<name>"}, "phases": {"triple_point_k": {"v": <n>, "unit": "K", "src": "<source id>"}, "triple_point_pa": {"v": <n>, "unit": "Pa", "src": "<source id>"}, "boiling_point_k": {"v": <n>, "unit": "K", "src": "<source id>", "at_pa": <pressure it was measured at, Pa>}, "critical_point_k": {...}, "critical_point_pa": {...}, "density_solid": {"v": <n>, "unit": "kg m-3", "src": "<source id>"}}, "optical": {"refractivity_589nm": {"v": <n>, "unit": "1", "src": "<source id>", "at_k": <K>, "at_pa": <Pa>}}}], "sources": [{"id": "<source id>", "text": "<authors year, journal or book: what it gives>", "doi": "<doi>" | "url": "<url>", "status": "unverified"}], "fictional": false}',
      'A value you could not read from its source carries "check": "unverified"; a stand-in carries "check": "authored" and cites "fudge-register". Leave out a property rather than guess it. Use a source already on the sheet where one fits (need sheet:sources); add a source row only for a new one. Invented matter: "fictional": true, and its values cite a source row that says it is invented for this setting.',
    ].join("\n"),
    async check(doc) {
      if (!obj(doc) || !Array.isArray(doc.substances) || !doc.substances.length) return { problems: ["a substance draft is {\"substances\": [rows], \"sources\": [rows]}"] };
      const d = doc as unknown as SubstanceDraft;
      const problems: string[] = [];
      const known = new Map(rowsOf(session, "substances").map((r) => [r.id, r]));
      for (const r of d.substances) {
        if (!obj(r) || typeof r.id !== "string" || !r.id) problems.push("every substance row needs an id");
        else if (known.has(r.id)) problems.push(`${r.id} is already on the data sheet (${nameOf(known.get(r.id)!)}): use it, or give the new one another id`);
      }
      if (d.sources !== undefined && !Array.isArray(d.sources)) problems.push("sources is a list of source rows");
      if (problems.length) return { problems };
      const id = d.fictional ? FICTIONAL_REFERENCE_PACK : REFERENCE_PACK;
      const st = session.stack();
      const mine = st.added.find((p) => p.id === id);
      const sheet = JSON.parse(JSON.stringify(mine ? sheetOf(mine) : {})) as Record<string, unknown>;
      sheet.substances = [...listOf<Row>(sheet.substances), ...d.substances];
      if (d.sources?.length) sheet.sources = [...listOf<Row>(sheet.sources), ...d.sources];
      const pack = { format: "sse.pack/1", id, version: bump(mine?.version ?? "0.0.0"), kind: ["physics"], kernel_range: "^0.1", licence: "Apache-2.0", ...(d.fictional ? { fictional: true } : {}), provides: { ...(mine?.provides ?? {}), reference: sheet } } as unknown as PackDoc;
      const added = [...st.added.filter((p) => p.id !== id), pack];
      const t = await session.trial({ added }, async () => true);
      return t.ok ? { problems: [], value: { doc: d, added } } : { problems: t.issues.length ? t.issues : [t.report] };
    },
    summary: ({ doc: d }) => {
      const props = (r: Row) => {
        // A value is an object holding v or fit, at the top of the row or inside a group (phases, optical, ...).
        const isValue = (v: unknown) => obj(v) && ("v" in v || "fit" in v);
        const n = Object.entries(r).filter(([k, v]) => obj(v) && !["formula", "names"].includes(k)).reduce((a, [, v]) => a + (isValue(v) ? 1 : Object.values(v as object).filter(isValue).length), 0);
        return `${nameOf(r)} (${Object.entries((r.formula as Record<string, number>) ?? {}).map(([e, c]) => `${e[0]!.toUpperCase()}${e.slice(1)}${c === 1 ? "" : c}`).join("")}, ${n} ${n === 1 ? "property" : "properties"})`;
      };
      const srcs = d.sources?.length ? `, ${d.sources.length} new ${d.sources.length === 1 ? "source" : "sources"}` : "";
      return `${d.substances.map(props).join(", ")}${srcs}. ${d.fictional ? "Marked fictional: everything that reads it says so. " : ""}Accept adds it to your setting's data sheet, so the editing screens can offer it.`;
    },
    async accept({ added }) {
      const r = await stackPacks(session, ws, added);
      return r.status === "applied" ? null : r.status === "refused" ? r.reason : r.status === "error" ? r.message : "waiting for approval";
    },
  };
}

/** The data sheet in force, for the catalogue: one line per table, the full row on request. */
export function sheetEntries(session: ViewSession): Entry[] {
  const subs = rowsOf(session, "substances");
  const mixes = rowsOf(session, "mixtures");
  const elements = rowsOf(session, "elements");
  const sources = rowsOf(session, "sources");
  return [
    { key: "sheet:substances", line: `the data sheet's ${subs.length} substances (need it for the ids)`, full: subs.map((r) => `${r.id}: ${nameOf(r)}`).join("\n") || "none" },
    ...subs.map((r) => ({ key: `substance:${r.id}`, line: `substance ${nameOf(r)}`, full: JSON.stringify(r) })),
    { key: "sheet:mixtures", line: `the data sheet's ${mixes.length} mixtures (airs, brines and the like)`, full: mixes.map((r) => `${r.id}: ${nameOf(r)}, ${Object.keys((r.recipe as object) ?? {}).join(" ")}`).join("\n") || "none" },
    { key: "sheet:elements", line: `the ${elements.length} elements a formula may name`, full: elements.map((r) => `${r.id} ${String(r.symbol ?? "")} ${String(r.name ?? "")}`).join("\n") },
    { key: "sheet:sources", line: `the ${sources.length} sources values may cite`, full: sources.map((r) => `${r.id}: ${String(r.text ?? "")}`).join("\n") },
  ];
}

/** Orac's handbook: how SSE4 is built and extended, one passage per topic (orac-handbook.json). */
export function handbookEntries(): Entry[] {
  return handbook.passages.map((p) => ({ key: `guide:${p.id}`, line: p.line, full: p.text }));
}
