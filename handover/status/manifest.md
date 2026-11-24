# Status: ship manifest and crew

Owner thread: "Build ship manifest and crew roster". Covers: MC (track: what is aboard a ship, station or site).

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`. Stage order: `handover/design/SSE4-coming-together.md`.

Scope (Alex, 23 Nov): the manifest and crew are general, game-neutral tools now; aliens remake them in 4.1 and RPG support in 4.2. Character sheets from pack data, dice and table rolls, and sites you go inside stay parked for 4.2 (example packs design section 0).

| Package | State | Log |
|---|---|---|
| MC1: base manifest pack (`sse.base-manifest`: `aboard.*` kept kinds, life support rates per kind of being, cited); everyone and everything aboard as record actors (kinds `person` and `cargo`) with an `aboard.on` link; an Aboard panel (Crew and Manifest) on the Ships and stations card (people against crew limit and berths, the ship's own capacities, readiness, recycled share, life support days per store, cargo with load and move; crew with role, kind of being, awake or cold sleep or missing or dead, own age); session `addActor`, `setKept` | done | packages/MC1-manifest-and-crew.md |
| MC2: Known to on entries aboard: the shared three-state eye on each person and lot (Build and Run desks, as on the card), and "Show the crew / the manifest to the table" as one change with one Undo; changes are made at the campaign clock, where Known to reads them | done | packages/MC2-known-to-aboard.md |
| MC3: a construct's pictures ("Picture" on the Ships and stations card) with a selector: Plans and 3D model of the ship designed from its own parts (renderer ShipPlans, closed in on the hull; style, remix and overrides kept as `construct.design`), and up to four of the GM's own pictures (`picture.own`, raster data URIs scaled to 960 px, about 150 KB each); the one last chosen on Build is kept (`picture.shown`) and is what it opens on. On Explore and Run any view can be chosen but nothing is kept, and the look cannot change (ShipPlans `locked`); crew roles and cargo amounts are editable in play, the ship's fittings only on Build | done | packages/MC3-construct-picture.md |
| MC4: fuel gauges in the Manifest: reaction mass and jump fuel, now against full (`construct.propellant_t`, `construct.jump_fuel_t`, full when unset, the kernel's own default), set or topped up in play; `fuelNow(ship, kept)` gives the ship with its tanks at the kept levels for planning; `shipRecordOf` resolves a nav ship through the kernel | done (fields moved to kept values in MC5); voyages drawing the tanks down is the travel track's (asked) | packages/MC4-fuel-gauges.md |
| MC5: the ship's Log reads what is aboard: who and what came aboard (from where), left (for where), and a person's state while aboard (`aboardLog`, from the entries' own histories); fuel levels become kept values in the base manifest pack (`fuel.propellant_t`, `fuel.jump_t`, unit t), so each set, top-up or voyage is a dated log line with its change and reason | done | packages/MC5-log-aboard.md |

## Alex's rulings for this track (so they live in the repo, not only in the thread)

- 23 Nov 19:53: "do the manifest and crew work... that IS good"; SSE3 had crew and life support as text boxes, SSE4's is a general tracking tool; "it will get a makeover with aliens [4.1] and again in RPG [4.2]". Keep it light and never assume crew are human.
- 24 Nov 14:00, **context by desk**: "explore/run will allow manifest/crew/fuel updates but not allow changes to the ship (and the pic & the 3d model we create or they supply)... the edit tab is the tool to hide things". So: in play (Explore, Run) what is aboard changes (people, cargo, roles, states, amounts, fuel); the ship itself (facts, fittings such as recycling, design, pictures, default view) changes only on Build. `ShipPlans` takes `locked` on play screens.
- 24 Nov 14:15, **picture selector**: "an image selector, pic, 2d 3d whatever the player loads... they can hold the default image as the last selected - which may be the 3d model". Built as the general `Pictures` panel (views/pictures); 14:28 he wants the same on the body card ("same as a ship"), which the chart room thread is doing.
- 24 Nov 14:18 to 14:19, **stations**: generated from their description, land constructs the same; about two dozen shapes sized to population with bolt-ons; docking points by size; a cut-off where it becomes a megastructure because it then touches physics. Built by the renderer thread (StationPlans, PR 408), hosted in the same Pictures panel.
- 24 Nov 14:20, **fuel**: "fuel gauge (and an edit fuel) to top up, etc - yup go for it!" Built (MC4, MC5); voyages planning from and drawing down the tanks is the travel thread's, agreed with it.

## Open questions for Alex (each with the track's recommendation; built on the recommendation meanwhile)

1. The six crew questions of 23 Nov (aboard is one kind of thing for people and cargo; kinds of being never assumed human; own age that stops in cold sleep; life support from the stores aboard; damage only through readiness; players learn of entries through Known to): no answer yet; all built as recommended.
2. Should players see a ship's pictures once the ship is shown to them? Recommended yes, through the same Known to eye. Blocked on a player-side ship card, which does not exist yet (player screens show no construct card).

## Next

- After PR 423 (MC5) merges: nothing queued that needs no one else. Waiting on others: the travel thread plans voyages from `fuelNow` and draws the tanks down with `setKept` (agreed 24 Nov); the chart room puts `Pictures` on the body card.
- Candidates, in order: a player-side ship card (with the chart room) so question 2 can be built; crew limits by kind of being with FT2 (future tech); `tests/ui/manifest.e2e.mjs` into CI's e2e list (CI edits need Alex's own yes).
