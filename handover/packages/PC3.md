# PC3 and PC4: the card, the wall, Keep as postcard, and the visit

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
- `tests/ui/postcards.e2e.mjs` runs in CI after render.e2e (one line added to `ci.yml` with Alex's own yes, 25 Nov
  01:51: "yes you can test postcards - we need them for testing").

## PC4 part 2: the visit drawn live (25 Nov)
A postcard sent from a campaign, opened in someone else's browser, now draws its front live, and it is the sender's
picture: the same world from the same camera (design section 4 rule 4).
- `shell/src/postcards/visiting.ts`: a visiting kernel, started on the first visit that needs one (one per universe
  visited, for the page's life), on the GM seat of its own. The campaign's kernel is never written to.
- What Send now packs with a clip (`world.origin`, ui-contract `PostcardOrigin`): the universe (id and seed), the
  system, the address each clip node had, and every authored fact those bodies and the system hold that an edit may
  name, through `goesInOrigin` (never notes, descriptions, anything "gm", secrets or Known-to; an import's own
  bookkeeping fields stay home). Also `world.about` (the clip node the camera is about) and `world.age_s` (the star's
  age). Only bodies the seat sees go: the clip is already through the seat.
- The visit opens that universe on the visiting kernel and sets the origin's facts at the same addresses, as authored
  facts at time 0, so the kernel makes the same ground. First try without them drew a bare rock: the clip's few fields
  (name, orbit, mass, radius, day) leave out water and air.
- A postcard with no origin (none are made now) falls back to `clipAsSystem`: the clip laid into a fresh system of
  the visiting universe, its star on the system's own address; it needs `age_s`, and says so plainly without it.
- `views/body/src/surface.ts` keeps surfaces per kernel as well as per address (`surfaceKept(ctx, system, address)`),
  since a visit reads the same addresses on another kernel.
- The e2e pastes the file Send saved into a fresh browser and checks the front is drawn and matches the sender's
  (24 by 24 grey thumbnails, mean difference under 0.03).

## Not yet
- An address postcard (an untouched generated place) still draws only its stamp: nothing makes one yet, and its
  `root` is the kernel's root, not the seed a visit would open. Give it the universe's id and seed when Send learns
  to make one (a place in a generated system not adopted into the campaign).
- A visit of a postcard made in System 3D waits for PC2 part 2 (photographView); its pose is in the system view's
  drawing frame, not the globe's (renderer thread, 25 Nov).
- "Make a postcard" on a bookmark row is the Run desk's Bookmarks list (asked; `postcardBlocker` and `fixDate` are
  there for it). Band switch on the front waits for other bands on the globe. Pass-by-pass reasons wait for PC2.
