// Layouts on the board (PB13, playbook and switchboard design 7.4): a layout is a players' screen document
// (`sse.screen/1`), named "layout" here so that "screen" can mean the glass. The campaign keeps the layouts the GM
// makes (`workspace.layouts`, with the save); the shipped ones are files beside the shell. One editor makes and edits
// them: an arrangement, the boxes on it (each a part showing whatever is sent, with its own controls on or off, AT13),
// a look and a name. A new layout is the editor on a blank copy.
import type { ScreenDoc, ScreenItem } from "./screen.ts";

/** The layouts that ship beside the shell: a seat's `screens[0]` names one by file ("" is the Bridge). */
export const SHIPPED_LAYOUTS: readonly { file: string; label: string }[] = [
  { file: "", label: "Bridge (the players' chart)" },
  { file: "players.bridge-3d.screen.json", label: "Bridge in 3D" },
  { file: "players.survey.screen.json", label: "Tarn survey report" },
  { file: "players.land.screen.json", label: "Landing" },
];

/** The arrangements the editor offers, in words, each a screen layout with a fixed set of boxes. */
export const ARRANGEMENTS = {
  one: { words: "One picture", layout: "stack", areas: ["1"] },
  side: { words: "Picture and side panels", layout: "grid", areas: ["a", "b", "c", "d", "e", "f"] },
  four: { words: "Two by two", layout: "flow", areas: ["1", "2", "3", "4"] },
  report: { words: "Report page", layout: "page", areas: ["1", "2", "3", "4", "5", "6", "7", "8"] },
  blank: { words: "Blank", layout: "stack", areas: [] as string[] },
} as const satisfies Record<string, { words: string; layout: ScreenDoc["layout"]; areas: readonly string[] }>;
export type Arrangement = keyof typeof ARRANGEMENTS;

/** The arrangement a document is in, read from its layout and how many boxes it has. */
export function arrangementOf(doc: Pick<ScreenDoc, "layout" | "items">): Arrangement {
  if (!doc.items.length) return "blank";
  if (doc.layout === "grid") return "side";
  if (doc.layout === "page") return "report";
  if (doc.layout === "flow") return "four";
  return doc.items.length === 1 ? "one" : "report";
}

/** A layout's id from its name: `mine.<words>`, never a shipped one's, never one already taken. */
export function layoutIdFor(name: string, taken: readonly string[]): string {
  const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 40) || "layout";
  let id = `mine.${slug}`;
  for (let n = 2; taken.includes(id); n++) id = `mine.${slug}-${n}`;
  return id;
}

/** The same layout in another arrangement: its boxes keep their order and move onto the new areas; extra boxes go. */
export function rearrange(doc: ScreenDoc, to: Arrangement): ScreenDoc {
  const a = ARRANGEMENTS[to];
  const items = sorted(doc).slice(0, a.areas.length).map((it, i) => ({ ...it, area: a.areas[i]! }));
  return { ...doc, layout: a.layout, items };
}

/** The boxes in their order on the screen (the arrangement's area order). */
export function sorted(doc: Pick<ScreenDoc, "layout" | "items">): ScreenItem[] {
  const order = Object.values(ARRANGEMENTS).find((x) => x.layout === doc.layout)?.areas ?? [];
  const rank = (it: ScreenItem) => (order as readonly string[]).indexOf(it.area);
  return [...doc.items].sort((x, y) => rank(x) - rank(y));
}

/** Move the box at `from` to `to` (their order on the screen); the areas stay where they are, the boxes move. */
export function moveBox(doc: ScreenDoc, from: number, to: number): ScreenDoc {
  const boxes = sorted(doc);
  if (from < 0 || from >= boxes.length || to < 0 || to >= boxes.length || from === to) return doc;
  const areas = boxes.map((b) => b.area);
  const [b] = boxes.splice(from, 1);
  boxes.splice(to, 0, b!);
  return { ...doc, items: boxes.map((it, i) => ({ ...it, area: areas[i]! })) };
}

/** Swap two boxes (dragging one onto another). */
export function swapBoxes(doc: ScreenDoc, i: number, j: number): ScreenDoc {
  const boxes = sorted(doc);
  if (i === j || !boxes[i] || !boxes[j]) return doc;
  const areas = boxes.map((b) => b.area);
  [boxes[i], boxes[j]] = [boxes[j]!, boxes[i]!];
  return { ...doc, items: boxes.map((it, k) => ({ ...it, area: areas[k]! })) };
}

/** Add a box at the end, on the arrangement's next free area (`on`, else the one it is in); unchanged when it is full. */
export function addBox(doc: ScreenDoc, view: string, on: Arrangement = arrangementOf(doc)): ScreenDoc {
  const a = ARRANGEMENTS[on === "blank" ? "one" : on];
  const used = new Set(doc.items.map((i) => i.area));
  const free = a.areas.find((x) => !used.has(x));
  if (!free) return doc;
  return { ...doc, layout: a.layout, items: [...doc.items, { view, area: free }] };
}

/** Take the box at `i` away; the boxes after it move up. */
export function removeBox(doc: ScreenDoc, i: number): ScreenDoc {
  const boxes = sorted(doc);
  if (!boxes[i]) return doc;
  const areas = boxes.map((b) => b.area);
  boxes.splice(i, 1);
  return { ...doc, items: boxes.map((it, k) => ({ ...it, area: areas[k]! })) };
}

/** A blank copy to start a new layout from: the Bridge's look, its theme and symbols, no boxes. */
export function blankLayout(like: ScreenDoc, name: string, taken: readonly string[]): ScreenDoc {
  return { schema: "sse.screen/1", id: layoutIdFor(name, taken), label: name, audience: "players", layout: "stack", theme: like.theme, symbology: like.symbology, vocab: like.vocab, items: [] };
}

/** An editable copy of a layout (a shipped one gets a new `mine.` id, so it is never overwritten). */
export function copyLayout(doc: ScreenDoc, taken: readonly string[]): ScreenDoc {
  const mine = doc.id.startsWith("mine.");
  const c = structuredClone(doc) as ScreenDoc;
  delete c.rail;
  return mine ? c : { ...c, id: layoutIdFor(doc.label, taken), label: doc.label };
}

/** Layouts kept with a campaign, from a save: only screen documents with a `mine.` id, each once. */
export function layoutsFrom(j: unknown): ScreenDoc[] {
  if (!Array.isArray(j)) return [];
  const out: ScreenDoc[] = [];
  for (const d of j) if (d && typeof d === "object" && (d as ScreenDoc).schema === "sse.screen/1" && typeof (d as ScreenDoc).id === "string" && (d as ScreenDoc).id.startsWith("mine.") && Array.isArray((d as ScreenDoc).items) && !out.some((x) => x.id === (d as ScreenDoc).id)) out.push(d as ScreenDoc);
  return out;
}

/** Keep one layout in the campaign's list: replaces one of the same id, else adds it at the end. */
export function keepLayout(list: readonly ScreenDoc[], doc: ScreenDoc): ScreenDoc[] {
  const i = list.findIndex((d) => d.id === doc.id);
  return i < 0 ? [...list, doc] : list.map((d, k) => (k === i ? doc : d));
}
