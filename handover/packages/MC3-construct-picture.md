# MC3: a construct's picture, and what changes in play

Track MC. Owner thread "Build ship manifest and crew roster". Asked by Alex, 24 Nov 13:52: "ships and stations - in edit mode would you be able to add a ship image or show/select the pregen'd one - that is other thread scope but this UI needs to be built alongside those tools", then 14:00: "explore/run will allow manifest/crew/fuel updates but not allow changes to the ship (and the pic & the 3d model we create or they supply). So our UI needs to be context sensitive - the edit tab is the tool to hide things", then 14:15: "being able to see both on the screen would be great - an image selector, pic, 2d 3d whatever the player loads... they can hold the default image as the last selected - which may be the 3d model".

## What it does

- A "Picture" button on the Ships and stations card (beside Aboard and Log) opens the construct's pictures (`packages/views/manifest/src/Picture.svelte`), with a selector when there is more than one: "Plans", "3D model", and a thumbnail per own picture.
- A ship with a nav ship is designed from its own parts by the renderer's `ShipPlans` (closed in on the hull). Its numbers come from the kernel (`ship_at_world` with no worlds resolves a stock id or a pack ship); rules and styles from the base-ships pack. `ShipPlans` gained `start` ("chart" or "3d") and `onview` (told when the reader switches), both opt-in.
- On the Build desk (edit mode): style switch and Remix, the last design kept as `construct.design` (`{style, seed, overrides?}`); "Add a picture…" (up to four, `construct.pictures`: each scaled so the longer side is 960 px, WebP or JPEG, a raster data URI of about 150 KB at most, `ui-contract/src/picture.ts`); "Remove this picture"; and the view last chosen is kept as `construct.picture_shown` ("plans", "model" or "picture:<n>"), which is what the card opens on everywhere.
- On Explore and Run: any view can be chosen (held for the session, not kept), but nothing changes: `ShipPlans` is `locked` (no style or Remix; turning and framing stay), no adding or removing pictures.
- Context by desk, per Alex: in play, what is aboard changes (add, move, state, crew roles, cargo amounts); the ship itself (facts, recycling share, pictures, design, default view) changes only on Build.
- The three picture fields are drawn, not listed: `PICTURE_FIELDS` are left out of the card's fact list.

## Not done

- Players do not see the picture yet (the card section is GM only); showing it to seats goes with Known to on the fields.
- Fuel in the tanks is not tracked as a level on the ship (voyages assume full tanks); fuel loaded as a cargo lot can be changed in play. A kept fuel level drawn down by voyages is the travel track's, asked as a question.
- Uploaded 3D models wait for packs carrying files (ship design memory, 23 Nov).

## Checks

- `tests/ui/manifest.e2e.mjs` (by hand): Explore shows the plans without Remix or upload, amounts editable; Build remixes and keeps the design (not listed as a fact), chooses the 3D model and keeps that choice, takes an 1800 x 1000 PNG and keeps it at 960 x 533 (three to choose from, opens on it); Explore opens on that picture, can switch to the plans, offers nothing to change; Build removes the picture and falls back to the plans.
- `packages/ui-contract/test/picture.test.ts`; `npm run ui:check`, `npm run ui:test`; setup and voyage walks.
- No Rust change; no golden change. Screens: /mnt/project-files/sse4/handover-screens/MC3-screens/.
