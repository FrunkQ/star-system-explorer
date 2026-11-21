// Orac's targets for blocks made of blocks and for game tables (A2 part 4).
//
// - Boxes (J3): a flow of blocks used as one block. The draft goes into the
//   campaign's own boxes pack and the stack is tried on the scratch kernel,
//   which compiles the box (unknown atom, unwired port, a box inside itself,
//   too deep) and runs its own golden cases, refusing in its own words.
// - Class tables: a game's lookup table ("size 0 is under 800 km, size 1 is
//   800 to 2,400 km...") as a scheme of classes in the class atlas, the same
//   mechanism the Traveller pack's codes use. The kernel's load checks the
//   atlas: unknown axis, parent cycle, a class with no bands, an id defined
//   twice. Accepting stacks the pack like any pack the GM adds, chronicled.
import { stackPacks, type PackDoc, type ViewSession, type Workspace } from "../../ui-contract/src/index.ts";
import type { Entry, OracTarget } from "./orac.ts";

export const BOXES_PACK = "campaign.boxes";
export const TABLES_PACK = "campaign.tables";

export interface BoxDraft {
  format?: string;
  id: string;
  version?: string;
  inputs?: { name: string; kind: string }[];
  outputs: { name: string; from: string }[];
  nodes: { id: string; atom?: string; box?: string; params?: Record<string, unknown> }[];
  wires?: { from: string; to: string }[];
  promoted_params?: { name: string; node: string; param: string }[];
  explain?: { curious?: string; science?: string; formula?: string };
  references?: unknown[];
  tests?: { name: string; inputs: Record<string, unknown>; expect: Record<string, unknown> }[];
  fictional?: boolean;
}

export type Bound = { axis: string; core?: [number, number]; fringe?: [number, number]; is?: boolean; any_of?: string[] };
export interface ClassRow {
  id: string;
  scheme: string;
  kind: "base" | "modifier";
  parent?: string;
  region: Bound[];
  threshold?: number;
  names?: { physics?: string; plain?: string; story?: string };
  learn?: { summary?: string };
  source?: string;
  [more: string]: unknown;
}
export interface TableDraft {
  scheme: { id: string; question?: string; [more: string]: unknown };
  /** New axes this table reads, `[id, {field, scale, label, span?, unit?, divide_by?}]`. */
  axes?: [string, { field: string; scale: string; label?: string; span?: [number, number]; unit?: string; divide_by?: number; [more: string]: unknown }][];
  classes: ClassRow[];
}

const obj = (x: unknown): x is Record<string, unknown> => !!x && typeof x === "object" && !Array.isArray(x);
const bump = (v: string) => {
  const [a = 0, b = 0, c = 0] = v.split(".").map(Number);
  return `${a}.${b}.${c + 1}`;
};
const packsOf = (session: ViewSession): PackDoc[] => {
  const s = session.stack();
  return [...s.base, ...(s.campaign ? [s.campaign] : [])];
};
const otherOf = (p: PackDoc | undefined) => (p?.provides?.other ?? {}) as Record<string, unknown>;
const listOf = <T>(x: unknown): T[] => (Array.isArray(x) ? (x as T[]) : []);
const refusal = (r: Awaited<ReturnType<typeof stackPacks>>) => (r.status === "applied" ? null : r.status === "refused" ? r.reason : r.status === "error" ? r.message : "waiting for approval");

/** The added packs with the campaign pack `id` remade by `edit` (its provides), version bumped. */
function remake(session: ViewSession, id: string, edit: (provides: Record<string, unknown>) => Record<string, unknown>): PackDoc[] {
  const st = session.stack();
  const mine = st.added.find((p) => p.id === id);
  const provides = edit(JSON.parse(JSON.stringify(mine?.provides ?? {})));
  const pack = { format: "sse.pack/1", id, version: bump(mine?.version ?? "0.0.0"), kind: ["game-system"], kernel_range: "^0.1", licence: "Apache-2.0", provides } as unknown as PackDoc;
  return [...st.added.filter((p) => p.id !== id), pack];
}

const issues = (t: { ok: false; report: string; issues: string[] }) => (t.issues.length ? t.issues : [t.report]);

export function boxTarget(session: ViewSession, ws: Workspace): OracTarget<{ doc: BoxDraft; added: PackDoc[] }> {
  return {
    kind: "box",
    what: "A box: several physics blocks wired together and used as one block, with its own inputs, outputs and worked examples (tests). From outside it looks like an atom, so a flow node can use it with \"box\": \"<id>@^<major>\". SSE compiles it and runs its tests before you see it. Read the atoms first (need atom:<id>), and a shipped flow for wiring (need flow:<id>).",
    form: [
      '{"format": "sse.box/1", "id": "<lower-case.id>", "version": "1.0.0", "inputs": [{"name": "<in>", "kind": "<quantity kind>"}], "outputs": [{"name": "<out>", "from": "<node>.<port>" | "in.<in>"}], "nodes": [{"id": "<node>", "atom": "<atom id>"} | {"id": "<node>", "box": "<box id>@^1"}], "wires": [{"from": "in.<in>" | "<node>.<port>", "to": "<node>.<port>"}], "promoted_params": [{"name": "<p>", "node": "<node>", "param": "<atom param>"}], "explain": {"curious": "<one plain sentence>", "science": "<the physics>", "formula": "<formula>"}, "tests": [{"name": "<case>", "inputs": {"<in>": {"value": <n>, "unit": "<unit>"}}, "expect": {"<out>": {"value": <n>, "unit": "<unit>", "tol": <relative>}}}]}',
      "Every input port of every node must be wired. Give at least one test from a published value (a fact sheet, a paper), and cite it in references.",
    ].join("\n"),
    async check(doc) {
      if (!obj(doc)) return { problems: ["a box is one JSON object"] };
      const d = { format: "sse.box/1", version: "1.0.0", inputs: [], wires: [], ...(doc as object) } as unknown as BoxDraft;
      const problems: string[] = [];
      if (typeof d.id !== "string" || !d.id) problems.push("a box needs an id");
      if (!Array.isArray(d.nodes) || !d.nodes.length) problems.push("nodes must list at least one block");
      if (!Array.isArray(d.outputs) || !d.outputs.length) problems.push("outputs must name at least one output");
      const taken = new Set(packsOf(session).flatMap((p) => [...listOf<{ id: string }>(p.provides?.boxes), ...listOf<{ id: string }>(p.provides?.flows)].map((b) => b.id)));
      if (typeof d.id === "string" && taken.has(d.id)) problems.push(`there is already a box or flow ${d.id}`);
      if (problems.length) return { problems };
      const added = remake(session, BOXES_PACK, (pr) => ({ ...pr, boxes: [...listOf<BoxDraft>(pr.boxes), d] }));
      const t = await session.trial({ added }, async () => true);
      return t.ok ? { problems: [], value: { doc: d, added } } : { problems: issues(t) };
    },
    summary: ({ doc: d }) => {
      const ins = (d.inputs ?? []).map((i) => i.name).join(", ") || "no inputs";
      const outs = d.outputs.map((o) => o.name).join(", ");
      const n = d.tests?.length ?? 0;
      return `Box ${d.id}@${d.version}: ${ins} → ${outs}, from ${d.nodes.length} ${d.nodes.length === 1 ? "block" : "blocks"}; ${n ? `${n} worked ${n === 1 ? "example passes" : "examples pass"}` : "no worked examples yet"}${d.explain?.curious ? `. ${d.explain.curious}` : ""}. Accept adds it to the campaign's boxes, for any flow to use.`;
    },
    async accept({ added }) {
      return refusal(await stackPacks(session, ws, added));
    },
  };
}

export function tableTarget(session: ViewSession, ws: Workspace): OracTarget<{ doc: TableDraft; added: PackDoc[] }> {
  return {
    kind: "table",
    what: "A game table as classes: a rule-book lookup (a size code from diameter, a temperature band, a danger rating) becomes a scheme in the class atlas, one class per row, each a region over named axes that read the world's facts. Codes are then derived from physics and answer \"why\" like any other label; they update when the world changes. Use the axes in force where one fits (need axes), add an axis only for a fact none reads, and read a shipped scheme first (need scheme:<id>).",
    form: [
      '{"scheme": {"id": "<lower-case.id>", "question": "<the question the table answers>"}, "axes": [["<axis id>", {"field": "<fact>", "scale": "linear"|"log"|"fraction"|"category"|"flag", "label": "<words>", "span": [<lo>, <hi>], "unit": "<unit after divide_by>", "divide_by": <n>}]], "classes": [{"id": "<scheme id>/<row>", "scheme": "<scheme id>", "kind": "base", "region": [{"axis": "<axis id>", "core": [<lo>, <hi>], "fringe": [<lo>, <hi>]}], "threshold": 1.0, "names": {"physics": "<code>", "plain": "<words>"}, "learn": {"summary": "<one plain sentence>"}, "source": "<rule book, edition, page>"}]}',
      "One base class per row; rows of one scheme must not overlap. core is the row's own range, fringe a little wider (or equal for hard table edges). A category axis uses \"any_of\": [\"<value>\", ...] and a flag axis \"is\": true|false in place of core and fringe. Copy table numbers exactly and name the book in source.",
    ].join("\n"),
    async check(doc) {
      if (!obj(doc)) return { problems: ["a table is one JSON object with scheme and classes"] };
      const d = doc as unknown as TableDraft;
      const problems: string[] = [];
      if (!obj(d.scheme) || typeof d.scheme.id !== "string" || !d.scheme.id) problems.push("scheme needs an id");
      if (!Array.isArray(d.classes) || !d.classes.length) problems.push("classes must list at least one row");
      if (d.axes !== undefined && !Array.isArray(d.axes)) problems.push("axes is a list of [id, axis] pairs");
      if (problems.length) return { problems };
      for (const c of d.classes) if (obj(c) && c.scheme !== d.scheme.id) problems.push(`class ${String(c.id)} is filed under scheme ${String(c.scheme)}, not ${d.scheme.id}`);
      if (problems.length) return { problems };
      const added = remake(session, TABLES_PACK, (pr) => {
        const o = (pr.other ?? {}) as Record<string, unknown>;
        return { ...pr, other: { ...o, schemes: [...listOf(o.schemes), d.scheme], axes: [...listOf(o.axes), ...(d.axes ?? [])], classes: [...listOf(o.classes), ...d.classes] } };
      });
      const t = await session.trial({ added }, async () => true);
      return t.ok ? { problems: [], value: { doc: d, added } } : { problems: issues(t) };
    },
    summary: ({ doc: d }) => {
      const axes = [...new Set(d.classes.flatMap((c) => c.region.map((b) => b.axis)))].join(", ");
      const rows = d.classes.map((c) => c.names?.physics || c.id.split("/").pop()).join(", ");
      const fresh = d.axes?.length ? `; new ${d.axes.length === 1 ? "axis" : "axes"} ${d.axes.map(([id]) => id).join(", ")}` : "";
      return `${d.scheme.question || d.scheme.id}: ${d.classes.length} ${d.classes.length === 1 ? "row" : "rows"} (${rows}) over ${axes}${fresh}. Accept adds it to the class atlas, so the codes are worked out from each world's facts.`;
    },
    async accept({ added }) {
      return refusal(await stackPacks(session, ws, added));
    },
  };
}

/** Boxes, axes and schemes in force, for the catalogue. */
export function blockEntries(session: ViewSession): Entry[] {
  const packs = packsOf(session);
  const boxes = packs.flatMap((p) => listOf<BoxDraft>(p.provides?.boxes));
  const axes = packs.flatMap((p) => listOf<[string, { field: string; scale: string; label?: string; unit?: string }]>(otherOf(p).axes));
  const schemes = packs.flatMap((p) => listOf<{ id: string; question?: string }>(otherOf(p).schemes));
  const classes = packs.flatMap((p) => listOf<ClassRow>(otherOf(p).classes));
  const row = (c: ClassRow) => `${c.id} (${c.kind}${c.parent ? `, under ${c.parent}` : ""}): ${c.region.map((b) => `${b.axis} ${b.core ? `${b.core[0]} to ${b.core[1]}` : b.any_of ? `one of ${b.any_of.join(", ")}` : `is ${JSON.stringify(b.is)}`}`).join(", ")}${c.names?.plain ? ` "${c.names.plain}"` : ""}`;
  return [
    ...boxes.map((b) => ({ key: `box:${b.id}`, line: `box ${b.id}@${b.version}: ${(b.inputs ?? []).map((i) => i.name).join(", ")} → ${b.outputs.map((o) => o.name).join(", ")}`, full: JSON.stringify(b) })),
    { key: "axes", line: axes.length ? `the class atlas axes: ${axes.map(([id]) => id).join(", ")}` : "no class atlas axes in force yet", full: axes.map(([id, a]) => `${id}: reads ${a.field}, ${a.scale}${a.unit ? `, in ${a.unit}` : ""}${a.label ? ` (${a.label})` : ""}`).join("\n") || "none" },
    ...schemes.map((s) => ({ key: `scheme:${s.id}`, line: `class scheme ${s.id}${s.question ? `: ${s.question}` : ""}`, full: [JSON.stringify(s), ...classes.filter((c) => c.scheme === s.id).map(row)].join("\n") })),
  ];
}
