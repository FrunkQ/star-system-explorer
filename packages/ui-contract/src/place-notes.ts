// A description and GM notes for the whole campaign and for each system (V3 gap 42, accepted by Alex 24 Nov 2026).
// V3 kept a map's description and GM notes on a floating panel, edited in place. SSE4 keeps them where every other
// note lives: in the campaign record (kernel `add_note`, record design section 2), at the place's own address (the
// universe's root for the campaign, the system's address for a system). Nothing new underneath:
//  - a note is never edited: saving a change writes a new note that supersedes the last, and clearing one writes an
//    empty note that withdraws it, so the record keeps the history and Undo-by-history works;
//  - the description is for everyone (tag "description", the same tag Describe uses on a body, V3 gap 23);
//  - GM notes are for the game master only (tag "gm-notes", audience "gm");
//  - notes never change the universe's state hash.
import type { RecordRead } from "./constructs.ts";

export type PlaceNoteKind = "description" | "gm-notes";

/** The tag and audience of each kind of place note. */
export const PLACE_NOTE: Record<PlaceNoteKind, { tag: string; audience: "everyone" | "gm"; words: string; hint: string }> = {
  description: { tag: "description", audience: "everyone", words: "Description", hint: "What this place is, as the table may come to know it" },
  "gm-notes": { tag: "gm-notes", audience: "gm", words: "GM notes", hint: "For you alone: plans, secrets, reminders" },
};

/** A live note of one kind at a place: its id (to supersede) and text. */
export interface PlaceText {
  id: string;
  text: string;
  at_s: number;
}

/** The universe's root address, where campaign-wide notes live ("u:kestrel/sys:1/body:3" → "u:kestrel"). */
export function campaignAddress(address: string): string {
  return address.split("/")[0]!;
}

/**
 * The current note of a kind at a place, from the record's live notes (the kernel has already dropped superseded
 * and withdrawn ones): the newest carrying the kind's tag, or null.
 */
export function placeNoteOf(notes: RecordRead["notes"], kind: PlaceNoteKind): PlaceText | null {
  const tag = PLACE_NOTE[kind].tag;
  let best: PlaceText | null = null;
  for (const n of notes) {
    if (!n.tags?.includes(tag)) continue;
    // Ties at the same universe time: the later in the list (fold order) wins.
    if (!best || n.at_s >= best.at_s) best = { id: n.id, text: n.text, at_s: n.at_s };
  }
  return best;
}

/** What to write to change a place note to `text`: an `addNote` request, or null when nothing changes. */
export function placeNoteEdit(
  address: string,
  kind: PlaceNoteKind,
  current: PlaceText | null,
  text: string,
): { address: string; text: string; audience: "everyone" | "gm"; tags: string[]; supersedes?: string } | null {
  const t = text.replace(/\s+$/, "");
  if (t === (current?.text ?? "")) return null;
  // An empty note with nothing to withdraw writes nothing (the record refuses empty text without a supersedes).
  if (!t.trim() && !current) return null;
  const { tag, audience } = PLACE_NOTE[kind];
  return { address, text: t.trim() ? t : "", audience, tags: [tag], ...(current ? { supersedes: current.id } : {}) };
}
