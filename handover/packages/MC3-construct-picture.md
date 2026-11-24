# MC3: a construct's picture, and what changes in play

Track MC. Owner thread "Build ship manifest and crew roster". Asked by Alex, 24 Nov 13:52: "ships and stations - in edit mode would you be able to add a ship image or show/select the pregen'd one - that is other thread scope but this UI needs to be built alongside those tools", then 14:00: "explore/run will allow manifest/crew/fuel updates but not allow changes to the ship (and the pic & the 3d model we create or they supply). So our UI needs to be context sensitive - the edit tab is the tool to hide things".

## What it does

- A "Picture" button on the Ships and stations card (beside Aboard and Log) opens the construct's picture (`packages/views/manifest/src/Picture.svelte`).
- A ship with a nav ship shows itself designed from its own parts: the renderer's `ShipPlans` (plans, 3D model, view controls), closed in on the hull. The ship's numbers come from the kernel (`ship_at_world` with no worlds resolves a stock id or a pack ship); rules and styles from the base-ships pack.
- On the Build desk (edit mode) the style switch and Remix are offered and the last design is kept with the construct as the fact `construct.design` (`{style, seed, overrides?}`, renderer DesignRecipe without abilities). The GM can give it a picture of their own instead: scaled so the longer side is 960 px, written as WebP (JPEG where WebP cannot be written), kept as `construct.picture` (a raster data URI only, no SVG; at most about 150 KB, `ui-contract/src/picture.ts`). "Use the designed ship" (or "Take the picture away" for a station or site) sets it to "".
- On Explore and Run the look is fixed: `ShipPlans` takes a new `locked` prop (renderer view, default off) that hides style and Remix; turning and framing stay. No upload.
- Context by desk, per Alex: in play, what is aboard changes (add, move, state, crew roles, cargo amounts); the ship itself (facts, recycling share, picture, design) changes only on Build.
- Both picture fields are drawn, not listed: `PICTURE_FIELDS` are left out of the card's fact list.

## Not done

- Players do not see the picture yet (the card section is GM only); showing it to seats goes with Known to on the fields.
- Fuel in the tanks is not tracked as a level on the ship (voyages assume full tanks); fuel loaded as a cargo lot can be changed in play. A kept fuel level drawn down by voyages is the travel track's, asked as a question.
- Uploaded 3D models wait for packs carrying files (ship design memory, 23 Nov).

## Checks

- `tests/ui/manifest.e2e.mjs` (by hand): Explore shows the designed ship without Remix or upload, amounts editable; Build remixes and keeps the design (not listed as a fact), takes an 1800 x 1000 PNG and keeps it at 960 x 533, and goes back to the designed ship.
- `packages/ui-contract/test/picture.test.ts`; `npm run ui:check`, `npm run ui:test`; setup and voyage walks.
- No Rust change; no golden change. Screens: /mnt/project-files/sse4/handover-screens/MC3-screens/.
