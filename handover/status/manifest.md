# Status: ship manifest and crew

Owner thread: "Build ship manifest and crew roster". Covers: MC (track: what is aboard a ship, station or site).

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`. Stage order: `handover/design/SSE4-coming-together.md`.

Scope (Alex, 23 Nov): the manifest and crew are general, game-neutral tools now; aliens remake them in 4.1 and RPG support in 4.2. Character sheets from pack data, dice and table rolls, and sites you go inside stay parked for 4.2 (example packs design section 0).

| Package | State | Log |
|---|---|---|
| MC1: base manifest pack (`sse.base-manifest`: `aboard.*` kept kinds, life support rates per kind of being, cited); everyone and everything aboard as record actors (kinds `person` and `cargo`) with an `aboard.on` link; Manifest and Crew panels on the Ships and stations card (people against crew limit and berths, the ship's own capacities, readiness, recycled share, life support days per store, cargo with load and move; crew with role, kind of being, awake or cold sleep or missing or dead, own age); session `addActor`, `setKept` | done (this track); entries in the ship's log, the Known to eye on entries, and the template crew figure joined to the roster, next | packages/MC1-manifest-and-crew.md |
