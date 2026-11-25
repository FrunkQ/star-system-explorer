// The GM's own copy (track AT, AT10; atlas design section 5, last paragraph). Seen by the GM, nothing is barred: what
// the table has not been told is in pencil, as on the chart (dotted underline, quieter ink), and that prints. The GM's
// notes for the place print in their own section. A GM briefing is therefore simply the Atlas seen by the GM.
//
// Pure. "What the table knows" is the table's own scene, the same bytes the table is sent: a line is in pencil exactly
// when the table's page would have a gap there (or the table does not know the thing at all). Nothing is guessed.
import { showFact } from "./catalogue.ts";
import { editionWord, type Edition } from "./edition.ts";
import type { AtlasPage, Block } from "./page.ts";
import type { PlayerScene, Vocab } from "../player-scene.ts";

export interface GmCopyOptions {
  vocab: Vocab;
  /** The table's own scene of the same place, when read: what everyone has been sent. Null leaves nothing in pencil. */
  table: PlayerScene | null;
  /** The GM's notes at this page's address (the record's live `gm-notes`), newest last. */
  gmNotes?: readonly string[];
}

/** Whether the table has been told this line of this thing: its own page would show it, not a gap. */
export function tableKnows(table: PlayerScene, address: string, field: string, vocab: Vocab): boolean {
  const s = table.subjects.find((x) => x.address === address);
  if (!s) return false;
  return showFact(s, field, { vocab }).state !== "gap";
}

/**
 * The GM's copy of a page built from the GM's scene: lines and table cells the table has not been told are marked
 * `pencil`, the page says once what pencil means, and the GM's notes come in their own section. A page that is not
 * the GM's is returned as it was: pencil is never drawn on anyone else's copy.
 */
export function withGmCopy(page: AtlasPage, audience: string, edition: Edition, opts: GmCopyOptions): AtlasPage {
  if (audience !== "gm") return page;
  const t = opts.table;
  const whole = t ? !t.subjects.some((x) => x.address === page.address) : false;
  let pencilled = false;
  const blocks = page.blocks.map((b): Block => {
    if (!t) return b;
    if (b.type === "title" && whole) return (pencilled = true), { ...b, pencil: true };
    if (b.type === "facts") {
      let touched = false;
      const rows = b.rows.map((r) => {
        if (r.state === "gap" || tableKnows(t, page.address, r.field, opts.vocab)) return r;
        touched = true;
        return { ...r, pencil: true };
      });
      if (!touched) return b;
      pencilled = true;
      return { ...b, rows };
    }
    if (b.type === "table") {
      const fields = b.fields;
      if (!fields) return b;
      let touched = false;
      const rows = b.rows.map((row) => ({
        ...row,
        cells: row.cells.map((c, j) => {
          if (c.state === "gap" || tableKnows(t, row.link, fields[j]!, opts.vocab)) return c;
          touched = true;
          return { ...c, pencil: true };
        }),
      }));
      if (!touched) return b;
      pencilled = true;
      return { ...b, rows };
    }
    return b;
  });
  const notes = (opts.gmNotes ?? []).map((x) => x.trim()).filter(Boolean);
  if (!pencilled && !notes.length) return page;
  const add: Block[] = [];
  if (pencilled) {
    add.push({
      type: "note",
      pencil: true,
      text: whole
        ? editionWord(edition, "note.pencil_whole", "In pencil: the table does not know this place yet.")
        : editionWord(edition, "note.pencil", "In pencil: what the table has not been told."),
    });
  }
  if (notes.length) add.push({ type: "gmnotes", title: editionWord(edition, "section.gm_notes", "GM notes"), lines: notes });
  // Before the notes of How we know and the margin, which end the page body.
  const at = blocks.findIndex((b) => b.type === "sources" || b.type === "margin");
  if (at < 0) blocks.push(...add);
  else blocks.splice(at, 0, ...add);
  return { ...page, blocks };
}
