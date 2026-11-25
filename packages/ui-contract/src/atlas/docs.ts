// Where the Atlas's documents come from in a pack stack: `provides.other.atlas`, three lists of whole documents.
// A shipped pack, a store pack and the campaign's own pack (Orac's accepted drafts, `campaign.atlas`) all use it.
import type { PackDoc } from "../physics.ts";
import type { EditionDoc } from "./edition.ts";
import type { NoteSet } from "./margin.ts";
import type { KitDoc } from "./kit.ts";

export interface AtlasDocs {
  editions: EditionDoc[];
  note_sets: NoteSet[];
  kits: KitDoc[];
  /** Sentences for entries that are not documents at all; the checks (checkEdition, checkNoteSet, checkKit) judge the rest. */
  refused: string[];
}

const LISTS = ["editions", "note_sets", "kits"] as const;

/**
 * Every edition, note set and interface kit the stack provides, in stack order. A later pack's document replaces an
 * earlier one with the same id (as a campaign pack stacks over a shipped one), keeping the earlier one's place.
 */
export function atlasDocsOf(packs: readonly PackDoc[]): AtlasDocs {
  const out: AtlasDocs = { editions: [], note_sets: [], kits: [], refused: [] };
  for (const p of packs) {
    const other = (p.provides as { other?: Record<string, unknown> } | undefined)?.other;
    const atlas = other?.atlas as Record<string, unknown> | undefined;
    if (atlas === undefined) continue;
    if (typeof atlas !== "object" || atlas === null || Array.isArray(atlas)) {
      out.refused.push(`The pack ${p.id} gives provides.other.atlas, which should be an object with editions, note_sets and kits.`);
      continue;
    }
    for (const k of Object.keys(atlas)) {
      if (!(LISTS as readonly string[]).includes(k)) out.refused.push(`The pack ${p.id} gives atlas.${k}; the lists are editions, note_sets and kits.`);
    }
    for (const list of LISTS) {
      const v = atlas[list];
      if (v === undefined) continue;
      if (!Array.isArray(v)) {
        out.refused.push(`The pack ${p.id} gives atlas.${list} that is not a list.`);
        continue;
      }
      const into = out[list] as { id: string }[];
      v.forEach((d, i) => {
        const id = (d as { id?: unknown } | null)?.id;
        if (typeof d !== "object" || d === null || typeof id !== "string" || !id) {
          out.refused.push(`Entry ${i + 1} of atlas.${list} in the pack ${p.id} has no id.`);
          return;
        }
        const at = into.findIndex((x) => x.id === id);
        if (at >= 0) into[at] = d as { id: string };
        else into.push(d as { id: string });
      });
    }
  }
  return out;
}
