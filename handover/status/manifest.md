# Status: ship manifest and crew

Owner thread: "Build ship manifest and crew roster". Covers: MC (track: what is aboard a ship, station or site).

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`. Stage order: `handover/design/SSE4-coming-together.md`.

Scope (Alex, 23 Nov): the manifest and crew are general, game-neutral tools now; aliens remake them in 4.1 and RPG support in 4.2. Character sheets from pack data, dice and table rolls, and sites you go inside stay parked for 4.2 (example packs design section 0).

| Package | State | Log |
|---|---|---|
| MC1: base manifest pack (`sse.base-manifest`: `aboard.*` kept kinds, life support rates per kind of being, cited); everyone and everything aboard as record actors (kinds `person` and `cargo`) with an `aboard.on` link; an Aboard panel (Crew and Manifest) on the Ships and stations card (people against crew limit and berths, the ship's own capacities, readiness, recycled share, life support days per store, cargo with load and move; crew with role, kind of being, awake or cold sleep or missing or dead, own age); session `addActor`, `setKept` | done (this track); entries in the ship's log next | packages/MC1-manifest-and-crew.md |
| MC2: Known to on entries aboard: the shared three-state eye on each person and lot (Build and Run desks, as on the card), and "Show the crew / the manifest to the table" as one change with one Undo; changes are made at the campaign clock, where Known to reads them | done | packages/MC2-known-to-aboard.md |
| MC3: a construct's picture ("Picture" on the Ships and stations card): the ship designed from its own parts (renderer ShipPlans, closed in on the hull), its style, remix and overrides kept as `construct.design`; or the GM's own picture (`construct.picture`, raster data URI scaled to 960 px, about 150 KB at most); both changed only on the Build desk (ShipPlans `locked` elsewhere). In play (Explore, Run) crew roles and cargo amounts are editable; the ship's fittings (recycling, facts, look) only on Build | done | packages/MC3-construct-picture.md |
