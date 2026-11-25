# PC1: the camera on the bookmark, and the `sse.postcard/1` envelope

Thread "Build postcards and hold backdrops", 25 Nov 2026. Design `design/SSE4-postcards-design.md` sections 3, 4, 7
and 12A (Alex's rulings, which win). Golden-neutral: nothing here reaches the kernel or the state hash.

## What
- `ui-contract/src/pose.ts`: `Pose` (about, eye as a body-frame unit direction plus a distance in metres, up,
  `fov_deg` across the kept box's height, `exposure { mode: long|eye, ev: 0 }`, `keep` box), stored the way the
  renderer thread said it builds its camera (design section 14). `checkPose`, `poseFrom` (lenient; directions made
  unit and up made square), `poseOfGlobe` / `globeOfPose` (the globe's turn and tip, matching the renderer's
  `viewRotation`), `fovToFit` (the kept box fits any screen shape).
- `ui-contract/src/bookmark.ts` (agreed with the Run desk thread): optional `pose` on `Bookmark` and `Whereabouts`;
  `makeBookmark` keeps it; `bookmarksFrom` drops a malformed pose, never the bookmark.
- `ui-contract/src/postcard.ts`: the envelope (bookmark with a fixed date and an instrument that can draw a still,
  world `address | clip | here`, credit, stamp, and one sending's message). `checkPostcard` refuses with one plain
  line and drops a bad stamp, caption or message instead. `isPostcard` (one store: a bookmark with a camera and a
  fixed date is a postcard in its own campaign), `postcardBlocker`, `fixDate`, `envelope`, `postcardHere`.
  `clipThroughSeat` (a clip cut to what a seat sees; a hidden body takes what is under it) and `goesWith` ("What goes
  with it" in plain words). `packsNeeded` (have, other version, free, paid) and `shopRefusal` / `forShop` (12A points
  3 and 4: the shop takes an address plus published packs only, never a clip, never a message). `makeMessage`
  (plain text, 280 letters, opens as "Wish you were here!"), `withMessage`, `MESSAGE_EXPIRED`. The ways it travels:
  `postcardText` (clipboard), `postcardLink` (`#postcard=` base64url, address only, never the stamp), `postcardFile`
  (`.ssepostcard.json`), `readPostcard` (JSON, a link or just its fragment), `looksLikePostcard`.
- `ui-contract/src/view.ts`: Workspace gains `camera` (the view in front keeps it current) and `aim` (a camera to go
  to once). `shell/src/App.svelte`: `whereabouts()` carries the camera, so "+ Bookmark where I am", Keep on Live and
  Update from live keep it; opening a bookmark sets `aim`. `views/body/src/Body.svelte`: the globe keeps
  `ws.camera` as `poseOfGlobe(...)` and takes an `aim` about its world once.

## Reference shots (agreed with the renderer thread, 25 Nov, for its MATCH1 and the base set)
- `pose.stretch?: "linear" | "asinh"` (the finish applies it with a composite honesty mark); `exposure.mode` stays
  long or eye until VP4 meters ("matched" comes then).
- `bookmark.view.mix` on a postcard (`PostcardView`, `MixEntry`: band, optional emission `line` H-alpha, O III or
  S II, weight, colour), read leniently by `mixFrom`. Kept off the shared `ViewChoice` until a view draws Composite.
- `reference?: { title, credit, licence, url?, taken_at? }` on the envelope: the famous picture a shot is compared
  against; the picture itself never travels.

## Choices made here
- Packs on a postcard are `{ id, version }` as on a clip, not the note's `range: "^4.0"`: one form for both, and
  `missingPacks`' rule (another version is named, not refused) carries over as `other-version`.
- A pose needs a radius to say its distance: the globe uses the air's radius, else the scene's radius; with neither
  it keeps no camera rather than invent one.

## Checks
- `node --test packages/ui-contract/test/postcard.test.ts` (12 tests) and `bookmark.test.ts`; with PC3: `npm run ui:check`,
  `npm run ui:test`, the bookmarks, globe and postcards e2e files, `audit-matter.py --closed`. No Rust changed.

## Next
- PC3 (the card, the wall, Keep as postcard), PC4 (the visit). The Run desk thread starts PC5 on this (showOf reads
  `bm.pose`).
