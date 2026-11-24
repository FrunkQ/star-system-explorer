// The edition loader's checks (track AT, AT3; atlas design section 12.6). Pure and DOM-free, so the pack loader, the
// Atlas and Orac (AT11, which sends these sentences back to the model as they are) all call the same thing.
// `checkEdition` runs checks 1 to 5; `testRender` is check 6. Each returns plain sentences; an empty list passes.
import { FIGURE_KINDS, readFigure } from "../figures.ts";
import type { PlayerScene, Vocab } from "../player-scene.ts";
import { isCatalogued } from "./catalogue.ts";
import { checkEditionShape, resolveEdition, PAGE_KINDS, SURVEY_EDITION, type EditionDoc } from "./edition.ts";
import { checkVoiceTable } from "./voice.ts";
import { atlasContents, buildPage, type AtlasPage, type FactRow } from "./page.ts";

/** Slots an edition's letterhead and print lines may use (design 12.4). */
export const EDITION_SLOTS: readonly string[] = ["brand.name", "brand.motto", "brand.logo", "audience", "date", "place", "chapter", "doc.number", "page", "pages"];

/** Limits (design 12.6 check 5). */
export const EDITION_LIMITS = { label: 80, description: 200, line: 120, linesPerPage: 200 } as const;

export interface EditionStack {
  /** Other editions in the pack stack (a parent named by `extends` must be here or shipped). */
  editions?: readonly EditionDoc[];
  /** Look ids that exist (player themes); absent: looks are not checked. */
  looks?: readonly string[];
  /** Vocabulary layers that exist; absent: not checked. */
  vocabs?: readonly string[];
  /** Margin note sets that exist; absent: not checked. */
  noteSets?: readonly string[];
}

/** Typed figures in a piece of text: a number with a unit the units engine reads ("5 kg", "0.7 bar", "71%"). */
export function typedFigures(text: string): string[] {
  const out: string[] = [];
  for (const m of text.matchAll(/[+−-]?\d[\d,]*(?:\.\d+)?(?:\s*×\s*10[⁰¹²³⁴⁵⁶⁷⁸⁹⁻]+)?\s*(?:%|[^\s\d.,;:!?(){}"]+)/g)) {
    if (FIGURE_KINDS.some((k) => readFigure(m[0], k).ok)) out.push(m[0]);
  }
  return out;
}

/** Every piece of text an edition carries, with where it is, for the text checks. */
function textsOf(d: EditionDoc): { where: string; text: string; slots: boolean }[] {
  const out: { where: string; text: string; slots: boolean }[] = [];
  for (const [k, v] of Object.entries(d.voice?.labels ?? {})) out.push({ where: `the label ${k}`, text: v, slots: false });
  if (d.voice?.gap !== undefined) out.push({ where: "the gap words", text: d.voice.gap, slots: false });
  if (d.voice?.gap_short !== undefined) out.push({ where: "the short gap words", text: d.voice.gap_short, slots: false });
  for (const [i, l] of (d.letterhead?.lines ?? []).entries()) out.push({ where: `letterhead line ${i + 1}`, text: l, slots: true });
  if (d.letterhead?.stamp) out.push({ where: "the stamp", text: d.letterhead.stamp, slots: true });
  if (d.print?.running_head) out.push({ where: "the running head", text: d.print.running_head, slots: true });
  if (d.print?.footer) out.push({ where: "the footer", text: d.print.footer, slots: true });
  for (const k of ["big", "title", "strap", "publisher"] as const) if (d.cover?.[k]) out.push({ where: `the cover's ${k} line`, text: d.cover[k]!, slots: k === "title" });
  for (const [name, t] of [["verdict", d.verdict], ["subtitle", d.subtitles]] as const) for (const [i, r] of (t?.rules ?? []).entries()) out.push({ where: `rule ${i + 1} of the ${name} table`, text: r.text ?? "", slots: false });
  if (d.verdict?.label) out.push({ where: "the verdict's label", text: d.verdict.label, slots: false });
  for (const k of ["label", "status"] as const) if (d.device?.[k]) out.push({ where: `the device's ${k} line`, text: d.device[k]!, slots: false });
  return out;
}

/**
 * Checks 1 to 5 of the edition loader: the schema; `extends` exists and does not loop; every field it names is in the
 * fact catalogue; no typed figure about a subject in any text; looks, vocabularies and note sets exist; sizes. Plain
 * sentences, each saying what to change; an empty list means the edition loads.
 */
export function checkEdition(doc: unknown, stack: EditionStack = {}): string[] {
  const shape = checkEditionShape(doc);
  if (shape) return [`${shape}.`];
  const d = doc as EditionDoc;
  const out: string[] = [];
  // 1. extends.
  try {
    resolveEdition(d.id, [...(stack.editions ?? []).filter((e) => e.id !== d.id), d]);
  } catch (e) {
    out.push(`${(e as Error).message}.`);
  }
  // 2. fields.
  for (const [kind, secs] of Object.entries(d.templates ?? {})) {
    for (const s of secs ?? []) {
      for (const f of [...(s.lines ?? []), ...(s.table?.lines ?? [])]) {
        if (!isCatalogued(f)) out.push(`The ${kind} template's section ${s.key} names "${f}", which is not a fact the Atlas knows. Use a field from the catalogue.`);
      }
    }
  }
  for (const k of Object.keys(d.voice?.labels ?? {})) {
    if (k.startsWith("line.") && !isCatalogued(k.slice(5))) out.push(`The label ${k} is for "${k.slice(5)}", which is not a fact the Atlas knows.`);
  }
  for (const [name, t] of [["verdict", d.verdict], ["subtitle", d.subtitles]] as const) if (t) out.push(...checkVoiceTable(name, t));
  // 3. no typed figures; slots only where slots go, and only known ones.
  for (const t of textsOf(d)) {
    for (const f of typedFigures(t.text)) out.push(`${t.where[0]!.toUpperCase()}${t.where.slice(1)} states a figure ("${f}"). An edition may not state a fact: leave figures to the page.`);
    for (const m of t.text.matchAll(/\{([^}]*)\}/g)) {
      if (!t.slots) out.push(`${t.where[0]!.toUpperCase()}${t.where.slice(1)} has a slot {${m[1]}}, but slots go only in the letterhead, stamp and print lines.`);
      else if (!EDITION_SLOTS.includes(m[1]!)) out.push(`${t.where[0]!.toUpperCase()}${t.where.slice(1)} has an unknown slot {${m[1]}} (known: ${EDITION_SLOTS.map((x) => `{${x}}`).join(", ")}).`);
    }
  }
  // 4. named things exist.
  if (d.look && stack.looks && !stack.looks.includes(d.look)) out.push(`The look "${d.look}" is not in the pack stack (there are ${stack.looks.join(", ")}).`);
  if (d.vocab && stack.vocabs && !stack.vocabs.includes(d.vocab)) out.push(`The vocabulary "${d.vocab}" is not in the pack stack (there are ${stack.vocabs.join(", ")}).`);
  for (const set of d.margin?.sets ?? []) if (stack.noteSets && !stack.noteSets.includes(set)) out.push(`The margin note set "${set}" is not in the pack stack.`);
  // 5. sizes.
  if (d.description.length > EDITION_LIMITS.description) out.push(`The description is ${d.description.length} characters; keep it to ${EDITION_LIMITS.description}, one line people choose by.`);
  for (const [k, v] of Object.entries(d.voice?.labels ?? {})) if (v.length > EDITION_LIMITS.label) out.push(`The label ${k} is ${v.length} characters; keep labels to ${EDITION_LIMITS.label}.`);
  for (const t of textsOf(d)) if (t.slots && t.text.length > EDITION_LIMITS.line) out.push(`${t.where[0]!.toUpperCase()}${t.where.slice(1)} is ${t.text.length} characters; keep it to ${EDITION_LIMITS.line}.`);
  for (const kind of PAGE_KINDS) {
    const n = (d.templates?.[kind] ?? []).reduce((a, s) => a + (s.lines?.length ?? 0) + (s.table?.lines.length ?? 0), 0);
    if (n > EDITION_LIMITS.linesPerPage) out.push(`The ${kind} page has ${n} lines; split it into chapters (at most ${EDITION_LIMITS.linesPerPage}).`);
  }
  return out;
}

/** Each known figure on a page by field, for comparing two editions of one page. */
function knownRows(p: AtlasPage): Map<string, FactRow> {
  const m = new Map<string, FactRow>();
  for (const b of p.blocks) if (b.type === "facts") for (const r of b.rows) if (r.state !== "gap") m.set(r.field, r);
  return m;
}

/**
 * Check 6, the test render: the edition is drawn for each given scene (a GM's and a seat's with partial knowledge) and
 * compared page by page with the Survey atlas. Every figure must match, and nothing may show for an audience that the
 * Survey atlas does not show them. `vocabs` gives each vocabulary layer by id ("plain" at least).
 */
export function testRender(doc: EditionDoc, scenes: readonly PlayerScene[], vocabs: Readonly<Record<string, Vocab>>, stack: EditionStack = {}): string[] {
  const out: string[] = [];
  let ed;
  let base;
  try {
    ed = resolveEdition(doc.id, [...(stack.editions ?? []).filter((e) => e.id !== doc.id), doc]);
    base = resolveEdition(SURVEY_EDITION.id);
  } catch (e) {
    return [`${(e as Error).message}.`];
  }
  const vocab = vocabs[ed.vocab] ?? vocabs.plain;
  if (!vocab) return ["The test render needs the plain vocabulary."];
  for (const scene of scenes) {
    for (const l of atlasContents(scene)) {
      const mine = buildPage(scene, l.address, ed, { vocab });
      const ref = buildPage(scene, l.address, base, { vocab: vocabs.plain ?? vocab });
      if (!mine || !ref) continue;
      const a = knownRows(mine);
      const b = knownRows(ref);
      for (const [f, r] of a) {
        const s = b.get(f);
        if (s && s.figure && r.text !== s.text) out.push(`On ${l.title}, as known to ${scene.audience}, ${f} reads "${r.text}" where the Survey atlas reads "${s.text}". An edition may change words, never a figure.`);
        if (!scene.subjects.find((x) => x.address === l.address)?.facts.some((x) => x.field === f && (x.value !== undefined || x.band))) {
          out.push(`On ${l.title}, ${f} shows for ${scene.audience}, who has not been given it.`);
        }
      }
    }
  }
  return out;
}
