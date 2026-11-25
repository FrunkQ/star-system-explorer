# PC5: grounds on holding and transmission cards, and the Quotes kind

Track PC (postcards), Run desk share. Design: `handover/design/SSE4-postcards-design.md` sections 8 and 12A (points 5
and 6), accepted by Alex 25 Nov 2026 01:27 UTC. Owner: "Build Run desk playbook and players' side".

## What it does
- **Quotes.** `sse.holding-cards/1` takes `kind: "quote"` with `by` (who said it). Cards are the same card by kind and
  words; `by` is not part of that, so a later pack's copy of a quote does not change who said it. The deck falls back
  to any other kind (plain, joke, quote), then "Stand by". The base pack has no quotes yet: the set is PC7 (Atlas).
- **A ground on the held card** (`board.ts`): `HoldCard` keeps its bare kinds and gains `{ kind, ground }` and
  `{ text, ground }` (a transmission card takes a ground by the same field). `HoldGround` is `"none"`, `{ bookmark }`,
  `{ postcard }` or `{ set }` ("base" is the universe's default set, "campaign" the campaign's own). Helpers:
  `holdKind`, `holdText`, `holdGround`, `withGround`, `withWords`, `holdCardFrom` (a playbook reads cards with it).
- **The line card's Ground row** (`run/GroundPick.svelte`): None, A postcard (a strip of the campaign's stamps),
  Postcards in turn, This campaign's postcards in turn. A change of kind or words keeps the ground; a click on a line
  holds with the kind and ground used last. A preset's Hold step has the same row.
- **The wire, per role** (`grounds.ts`, `boardWire`'s `ground`): the GM's machine picks the postcard and sends its
  kept picture (the stamp, a data URL) as `ground: { id, name, picture, caption? }` beside the card. A campaign
  postcard goes only to a role whose audience knows the place it looks at (asked of the kernel with
  `scene(scope, { audience })`, kept until the board changes or the postcards turn), else the next one in turn, else
  no ground. A set's postcard is somewhere else, so nothing is hidden. Postcards in turn move on every 3 minutes
  (`GROUND_TURN_MS`), never the same twice running.
- **One piece of words** (12A point 5): a held joke or quote goes with `kind` on the wire, so
  `waitingLineFits(shown)` is false and the Atlas banner picker adds no waiting line under it. Blank, plain and typed
  words may have one.
- **The held screen** (`Players.svelte`): the picture whole on a soft blurred wash of itself, the words low left over a
  shade, who said a quote beneath, the postcard's name small at the foot. A new postcard fades in; under reduced
  motion the screen keeps the first one while the same card is held. Only a `data:image/…;base64` picture is drawn.
- **A sent bookmark carries its camera**: `showOf` puts the bookmark's `pose` on `Shown.pose`.
- **Make a postcard** on a Bookmarks row: says why a bookmark cannot be one yet (`postcardBlocker`), else fixes its
  date (`fixDate`) and opens it on the wall (`openOwn` in postcards/visits). Postcard rows carry a small stamp mark.

## Fudges and limits
- The ground is the stamp the GM's machine kept, drawn as the GM saw it; the role's knowledge decides only whether it
  is shown (by the place it looks at). Drawing it live through the role's own knowledge and look waits on the still
  of any view (PC2) and a visiting session (PC4 part 2).
- Sealed seats (their own room) get the card's words but not the ground yet.
- The sets: `share.setGrounds({ bookmarks, sets })` takes the packs' postcard sets once PC6 reads them; until then
  "Postcards in turn" uses the campaign's own.

## Checks
- `packages/ui-contract/test/grounds.test.ts` (7): card shapes and reading, quotes and the fallback, one piece of
  words on the wire, grounds per role by knowledge and turn, sets, the camera on a sent bookmark.
- `board.test.ts`: a held joke now carries `kind: "joke"` (on purpose).
- `tests/ui/hold-grounds.e2e.mjs`: a postcard kept from Earth's globe; the Bookmarks stamp and "Make a postcard"; on
  air, a held screen of the table gets no ground while the table does not know Earth, then the postcard full screen
  once Earth is revealed; a quote is marked; ground None takes the picture away. Run locally with the switchboard,
  playbook, bookmarks, postcards, players and table browser tests (all pass, 25 Nov). It is not in CI yet: adding its
  line to ci.yml was held by a permission check, so it waits for Alex's own yes.
