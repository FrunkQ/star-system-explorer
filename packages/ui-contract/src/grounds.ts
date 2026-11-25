// Grounds on held screens (PC5, SSE4-postcards-design.md section 8 and 12A points 5 and 6). A held card may carry a
// ground: one postcard, or postcards in turn from a set. The GM's machine works out, per role, which postcard a held
// screen shows and sends its kept picture with the card, so the screen draws nothing of what it was showing. A
// campaign postcard is somewhere this campaign's roles may not know: a role sees it only if it knows the place the
// postcard looks at, else the next one in turn, else no ground. A postcard from a set is somewhere else entirely, so
// there is nothing to hide. Presentation only: never read by physics, never in the state hash.
import type { Bookmark } from "./bookmark.ts";
import type { HoldGround } from "./board.ts";
import { checkPostcard, isPostcard, type Postcard } from "./postcard.ts";
import type { HeldGround } from "./seat.ts";

/** How often postcards in turn change (design 8.2: "every few minutes"), by a slow fade on the screens. */
export const GROUND_TURN_MS = 180_000;

/** A postcard that could be a ground: its picture, and for a campaign one the place a role must know to see it. */
export interface GroundCandidate {
  id: string;
  name: string;
  picture: string;
  caption?: string;
  here?: { scope: string; about: string };
}

/** A set of postcards from a pack (`sse.postcards/1`, PC6): its id and its postcards. */
export interface GroundSet {
  id: string;
  cards: readonly Postcard[];
}

/** The campaign's postcards that have a kept picture (a postcard opened on the wall once keeps one). */
export function campaignGrounds(
  bookmarks: readonly Bookmark[],
  stamp: (scope: string, id: string) => string | null,
): GroundCandidate[] {
  const out: GroundCandidate[] = [];
  for (const b of bookmarks) {
    if (!isPostcard(b)) continue;
    const picture = stamp(b.scope, b.id);
    if (!picture) continue;
    out.push({
      id: b.id,
      name: b.name,
      picture,
      here: { scope: b.scope, about: b.pose?.about ?? b.selected ?? b.scope },
    });
  }
  return out;
}

/** A postcard from outside the campaign as a ground: only one that carries its stamp can be drawn on any screen. */
export function outsideGround(p: Postcard): GroundCandidate | null {
  if (!p.stamp || !/^data:image\//.test(p.stamp)) return null;
  return {
    id: p.id,
    name: p.name,
    picture: p.stamp,
    ...(p.caption ? { caption: p.caption } : {}),
    ...(p.world.how === "here"
      ? {
          here: {
            scope: p.bookmark.scope,
            about:
              p.bookmark.pose?.about ?? p.bookmark.selected ?? p.bookmark.scope,
          },
        }
      : {}),
  };
}

/**
 * The postcards a ground draws from, in turn order. `{set: "base"}` is the universe's default set (12A point 6): the
 * last set on the stack (a universe pack's own wins over the base app's), and the campaign's own postcards while no
 * pack carries one. `{set: "campaign"}` is the campaign's own; another id is that set.
 */
export function groundCandidates(
  g: HoldGround,
  src: { campaign: readonly GroundCandidate[]; sets: readonly GroundSet[] },
): GroundCandidate[] {
  if (g === "none") return [];
  const ofSet = (s: GroundSet) =>
    s.cards.map(outsideGround).filter((c): c is GroundCandidate => !!c);
  if ("bookmark" in g) return src.campaign.filter((c) => c.id === g.bookmark);
  if ("postcard" in g) {
    const r = checkPostcard(g.postcard);
    const c = r.ok ? outsideGround(r.postcard) : null;
    return c ? [c] : [];
  }
  if (g.set === "campaign") return [...src.campaign];
  if (g.set === "base") {
    for (let i = src.sets.length - 1; i >= 0; i--) {
      const c = ofSet(src.sets[i]!);
      if (c.length) return c;
    }
    return [...src.campaign];
  }
  const s = src.sets.find((x) => x.id === g.set);
  return s ? ofSet(s) : [];
}

/**
 * The ground one role's held screens show: the postcard at `turn` in the list (so it is never the same twice running
 * while there are two or more), or the next one the role may see. `knows` answers for a campaign place: true, false,
 * or null while it is being asked (not shown yet). Null: no ground.
 */
export function pickGround(
  list: readonly GroundCandidate[],
  turn: number,
  knows: (scope: string, about: string) => boolean | null,
): HeldGround | null {
  const n = list.length;
  for (let i = 0; i < n; i++) {
    const c = list[(((turn + i) % n) + n) % n]!;
    if (c.here && knows(c.here.scope, c.here.about) !== true) continue;
    return {
      id: c.id,
      name: c.name,
      picture: c.picture,
      ...(c.caption ? { caption: c.caption } : {}),
    };
  }
  return null;
}

/** Does this ground change on its own (postcards in turn)? The GM's machine turns them every `GROUND_TURN_MS`. */
export function groundTurns(g: HoldGround): boolean {
  return typeof g === "object" && "set" in g;
}
