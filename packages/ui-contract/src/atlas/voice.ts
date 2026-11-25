// The edition's voice beyond words for words (track AT, AT7; atlas design 6.5): a verdict stamp and a subtitle phrase,
// each a label read off real facts through a small table in the edition, in the spirit of "processes, not classes".
// Rules use the margin notes' condition language and read only what the audience knows; the first rule that is met
// wins, and nothing is said when none is. Each carries its Why: the facts it came from, said by the figures door.
import { conditionFields, meets, type NoteCondition } from "./margin.ts";
import { isCatalogued, lineLabel, showFact, type SayIn } from "./catalogue.ts";
import type { Edition, PageKind } from "./edition.ts";
import type { AtlasPage } from "./page.ts";
import type { PlayerScene } from "../player-scene.ts";

export interface VoiceRule {
  /** Page kinds the rule is for; absent, any. */
  about?: PageKind[];
  when: NoteCondition;
  text: string;
}

export interface VoiceTable {
  /** The stamp's lead word ("Verdict"); the subtitle has none. */
  label?: string;
  rules: VoiceRule[];
}

/** What a table said for a page, and why. */
export interface VoiceSaid {
  label?: string;
  text: string;
  /** "Air pressure: 1.3 bar", one per fact the rule read. */
  why: string[];
}

/** The first rule of the table met by the page's subject, as the audience knows it; null when none is. */
export function readTable(table: VoiceTable | undefined, scene: PlayerScene, page: AtlasPage, opts: SayIn, labels: Record<string, string> = {}): VoiceSaid | null {
  if (!table) return null;
  const s = scene.subjects.find((x) => x.address === page.address);
  if (!s) return null;
  for (const r of table.rules) {
    if (r.about && !r.about.includes(page.kind)) continue;
    if (!meets(s, r.when)) continue;
    const why = [...new Set(conditionFields(r.when))].map((f) => {
      const shown = showFact(s, f, opts);
      return `${lineLabel(labels, opts.vocab, f)}: ${shown.state === "gap" ? "" : shown.text}`;
    });
    return { ...(table.label ? { label: table.label } : {}), text: r.text, why };
  }
  return null;
}

/** The page with the edition's verdict and subtitle phrase on its title (unchanged when the edition has neither). */
export function withVoice(page: AtlasPage, scene: PlayerScene, ed: Edition, opts: SayIn): AtlasPage {
  if (!ed.verdict && !ed.subtitles) return page;
  const verdict = readTable(ed.verdict ?? undefined, scene, page, opts, ed.voice.labels);
  const phrase = readTable(ed.subtitles ?? undefined, scene, page, opts, ed.voice.labels);
  if (!verdict && !phrase) return page;
  return {
    ...page,
    blocks: page.blocks.map((b) =>
      b.type === "title" ? { ...b, ...(phrase ? { subtitle: `${b.subtitle} · ${phrase.text}`, phrase } : {}), ...(verdict ? { verdict } : {}) } : b,
    ),
  };
}

/** Why a voice table will not load, as sentences (loader check 2 for tables). */
export function checkVoiceTable(name: string, t: VoiceTable): string[] {
  const out: string[] = [];
  if (!Array.isArray(t.rules)) return [`The ${name} table has no list of rules.`];
  t.rules.forEach((r, i) => {
    if (!r || typeof r.text !== "string" || !r.text.trim()) out.push(`Rule ${i + 1} of the ${name} table has no words.`);
    if (!r?.when) out.push(`Rule ${i + 1} of the ${name} table has no "when": a ${name} is read off facts.`);
    for (const f of conditionFields(r?.when)) if (!isCatalogued(f)) out.push(`Rule ${i + 1} of the ${name} table tests "${f}", which is not a fact the Atlas knows.`);
  });
  return out;
}
