// Postcards opened from outside this page's campaign (a pasted postcard, a link, a file): visits, read-only (design
// section 4 rule 2). Kept for the page's life only; opening one never changes the campaign. "Keep" on a visit is the
// ordinary paste of its clip.
import type { Postcard } from "../../../ui-contract/src/index.ts";

export const visits = $state<{ list: Postcard[]; open: string | null; own?: string | null }>({ list: [], open: null, own: null });

/** Open one of this campaign's postcards on the wall (the Bookmarks row's "Make a postcard", PC5). */
export function openOwn(id: string): void {
  visits.own = id;
}

/** Open a postcard as a visit: it goes to the top of the list and is the one shown. */
export function visit(p: Postcard): void {
  visits.list = [p, ...visits.list.filter((x) => x.id !== p.id)].slice(0, 24);
  visits.open = p.id;
}

/** The events the wall sends the shell: go to a bookmark's place, or paste a clip (Keep this system). */
export const GO_BOOKMARK_EVENT = "sse:go-bookmark";
export const PASTE_CLIP_EVENT = "sse:paste-clip";
