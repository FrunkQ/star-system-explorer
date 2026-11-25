# PC3 and PC4 part 1: the card, the wall, Keep as postcard, and the visit

Thread "Build postcards and hold backdrops", 25 Nov 2026. Design `design/SSE4-postcards-design.md` sections 4 to 7
and 12A. Built on PC1 (`packages/PC1.md`). Golden-neutral.

## What
- `shell/src/postcards/` (this track's section folder):
  - `Wall.svelte`, screen `postcards` (`documents/postcards.gm.json`, rail group Look at every scale, never a tab;
    `screens.postcards.svelte.ts` spreads its view and screen into the registry and App): this campaign's postcards
    (bookmarks with a camera and a fixed date, one store), then "Opened here" (visits). A row of stamps; press one to
    open its card.
  - `Card.svelte`: front drawn live through the still tier at 1024 px with a refining line; Turn over, Go there,
    Send. Back: name, caption, "Why it looks like this" (for now the still's own note and the Readable-scale line;
    the pass-by-pass reasons come with PC2), Where, When (the campaign calendar in its own system, the system clock
    for a place elsewhere), How (style, band, scale, field of view, exposure in words), Who, Travels as, the packs it
    needs with their state (12A point 3), the newer-physics line, a visit's "Visiting: a postcard from …" and its
    message. Foot: Turn over, Go there, Keep this system (a visit that carries a clip: the ordinary paste), Send, Save
    picture (PNG).
  - `Send.svelte`: "Write on the back" opening as "Wish you were here!", From, "What goes with it", Copy, Copy link
    (address postcards only; the reason shown otherwise), Save file (`.ssepostcard.json`, with the stamp).
  - `pack.ts`: a campaign postcard is packed as a clip of the body in frame, what is under it and what it orbits, as
    the table sees it once the campaign is shared (kernel scene for audience "everyone"), else as the GM sees it.
  - `still.ts`: `stillBlocker` / `drawStill` (the globe through `photograph`; every other instrument says plainly it
    waits for PC2 and shows the stamp).
  - `visits.svelte.ts`: visits for the page's life, and the two window events the wall sends the shell.
- `ui-contract/src/postcard-stamps.ts`: stamps of campaign postcards kept on this device (48 at most), `stampOf`.
- `views/body/src/Body.svelte`: "Keep as postcard" beside the photograph just taken.
- `ui-contract/src/open-file.ts`: a `v4-postcard` kind, so the one Open door names it and opens it as a visit.
- `shell/src/App.svelte` (shared, small): the screen and view lists; the wall's Go there (`sse:go-bookmark`) and
  Keep this system (`sse:paste-clip`); a pasted postcard (any desk), a `#postcard=` link (start sheet skipped, the
  address bar drops it once open) and an opened file all open as a visit on the wall.

## Checks
- `npm run ui:check`, `npm run ui:test`, `audit-matter.py --closed`; e2e `postcards` (new, in `ui:e2e`), `bookmarks`,
  `globe`, `body-pictures`, `shell`, `render` in chromium. Screenshots in project files
  `sse4/handover-screens/PC-screens/build/`.
- `tests/ui/postcards.e2e.mjs` is not yet a line in `.github/workflows/ci.yml`: adding it was stopped by a session
  permission check, so it needs Alex's own yes (one line after render.e2e: `node tests/ui/postcards.e2e.mjs --screens
  ui-screens/postcards`).

## Not yet
- A visit of an address place not open here draws only its stamp: resolving another universe's system for a visit
  needs a visiting session (PC4 part 2). A clip visit is kept with Keep this system (the paste).
- "Make a postcard" on a bookmark row is the Run desk's Bookmarks list (asked; `postcardBlocker` and `fixDate` are
  there for it). Band switch on the front waits for other bands on the globe. Pass-by-pass reasons wait for PC2.
