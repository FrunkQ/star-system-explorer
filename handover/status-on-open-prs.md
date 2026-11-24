# Status rows that exist only on open PR branches (copied 24 Nov 2026 17:15 UTC)

These are the owner threads' own status files as they stand on open PR branches, copied here so they survive if
the PRs are still open at the account move. Once a PR below merges, its file in `handover/status/` is the truth and
the copy here is stale; delete this file when all three have merged.

---

## `handover/status/run-desk.md` from PR 421 (branch `claude/project-thread-n8ko07`)

### Status: Run desk and players' side

Owner thread: "Build Run desk playbook and players' side" (from 24 Nov 2026, taken over from the chart room). Covers: the players' page (packages/shell/src/Players.svelte, players.ts, documents/players.*.screen.json), the table's part of the Run desk (packages/shell/src/run/: TableStatus, SeatsPanel, ArrivalSlip), and from design the Playbook and Switchboard. Shared seams changed only with a note to the chart room: ui-contract seat.ts `Shown`, p2p host state (`shown`, `party`), `ws.sendLanding`. Tests owned: table.e2e, landing-table.e2e, party.e2e, players.e2e.

Only the owner thread edits this file. Earlier rows for this area (What the table is shown, adopted systems reach players, the table watches a landing, players see the party's ship, seat rows first cut) stay in `status/chart-room.md`.

| Package | State | Log |
|---|---|---|
| RD1 reveal on arrival and a landing sent too soon (landing run-through gap 6; landing-table race): when the party's ship comes to a world the table does not fully know, the table's line offers "Beowulf is at Mars. Tell the table about it?"; one press sends the ordinary `set_known` (audience everyone, landmark known, at the campaign's now) with Undo (`undo_known`) and "Not now". Logic in ui-contract arrival.ts (`arrivalOffer`, `arrivalReveal`). A "Send this view" pressed on Land before the Land screen's first landing arrived now goes again with the landing (it went as the plain system and the players never saw the landing). | done | packages/RD1.md |
| PB1 the board underneath (design SSE4-playbook-and-switchboard-design.md, accepted 24 Nov): ui-contract board.ts holds who sees what as plain data (Everyone as a layer over the floor, a line per role and per screen, holds with a card, Hold all that remembers, redirects, signal levels) with undo and redo (`BoardHistory`); `boardWire` turns it into the old `Shown` per role and per screen, so older players still work. Players' screens say hello with a name and a role (`?name=`, id kept per tab); a held screen shows its card ("Stand by", plain words or a joke). Sealed seats get an empty scene while held. | done | packages/PB1.md |
| PB2 the Switchboard screen (rail entry under Table, every scale): the Live card and Send, roles and screens as a list with lines, press a line to hold and again to release, Hold all (H), Lift, Back, Undo and Redo (Ctrl+Z), drag or press-then-press a source onto a role or screen, move and swap screens by their grip. The status line's table half is now one on-air chip that opens it; Send this view stays. Seats dialog stays behind "More seats" until PB7. | done | packages/PB2.md |
| PB2b Bookmarks beside Undo (needs the chart room's top bar) | next | |
| PB3 the Playbook; PB3b who steers (Theirs, Read only); PB4 to PB14 per design section 15 | queued | |

#### Next
- PB2b, then PB3 and on in design order. The on-air chip moves to the chart room's top bar when it has room. Small: the Run desk card at Curious leads with four facts and drops Mass for planets with a day and a year (gap 10): depth.ts puts mass fifth by design; ask Alex with the Switchboard questions rather than change it alone.

---

## `handover/status/traveller.md` from PR 400 (branch `claude/project-thread-mzdt73`)

### Status: Traveller pack

Owner thread: "Build Traveller pack". Covers: X3a to X3d, TG8, TG9, TG11, TG13, MP16.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`. Stage order: `handover/design/SSE4-coming-together.md`.

| Package | State | Log |
|---|---|---|
| X3a: code sheets (`sse-codes`) and the Traveller pack's world codes (UWP, PBG, trade codes, zones, bases) from classification rules; units, Imperial calendar (`ui-contract/calendar.ts`), words; kernel op `codes`; world data sheet view (`sse.view.world-codes`) | done (Traveller pack track); lens, reports and sifters next | packages/X3a.md, packages/X3a-sheet.md |
| X3b part 1: a whole system from a coded world (pins to a formation request, codes win, impossible values marked with reasons, make physically sound, counts searched) | done (Traveller pack track); kernel one-shot op and moons next | packages/X3b.md |
| X3c: hex map view (`sse.view.hex-map`); sector formats (`sse-sector-io`); import through the code sheet (kernel op `import_sector`, preview or authored events); Travellermap fetch through the site Worker (`/tmap/*`) | done (Traveller pack track); GM screens in X3e | packages/X3c.md, packages/X3c-io.md |
| X3d parts 1 to 3: gas mixes and what to wear outside; sensors, range bands and the knowledge ladder as triggers and grants (to each ship's own crew); stock ships, stations and sites as J7 templates | done (Traveller pack track); add from templates (gap 13), readiness (TG9), jump measure (MP16) next | packages/X3d.md |
| X3e: chart room screens: Sector map (galaxy scale: hex map, import with preview, export, sifters, reach) and Codes (world scale); kernel op `sector_map`; rail entries that `need` a pack key | done (Traveller pack track); the stage 5 walk as an e2e next | packages/X3e-screens.md |
| X3f: Ships and stations screen (V3 gap 13): constructs in a system, Add here from the stacked templates, cards by depth, edit in place, take out of play | done (Traveller pack track); constructs on the chart and flight profiles next | packages/X3f-constructs.md |
| TG9: `ship.readiness` and `ship.discipline` kept values with template defaults; `status.derelict` and `status.damaged` act only through visible pack triggers | done (Traveller pack track); programs that read them, and kept values on the card, next | packages/TG9-readiness.md |
| TG13: two universe packs written for SSE4 (`sse.near-future`, `sse.space-opera`: templates, factions, labels, dressing rules, a political lens); Traveller ports and bases as dressing rules; both on the Packs screen | done (Traveller pack track); ghost markers are TG14 (chart room) | packages/TG13-universe-packs.md |
| TG8 part 1: `access.*` labels in a base hooks pack (`sse.base-hooks`); the optional intrigue pack (`sse.intrigue`, seeded, GM-only, off by default) | done (Traveller pack track); part 2, `resource.*` over the TG11 inventory, waits on TG11 | packages/TG8-access-intrigue.md |
| V3 gap 16 part 1: the ship's log on the construct card, read from the history and the record (`shipLog`, session `record`) | done (Traveller pack track); g-force and dose on the voyage card (travel thread), drift (V8), and which facts an edit changed, next | packages/G16-ships-log.md |
| V3 gap 26: a seat's view inside a virtual tabletop (`&embed=1`, exact origin list on the seat, refusal with a reason, a ready and focus message bridge) | done (Traveller pack track); a "subject clicked" message when a tabletop asks | packages/G26-embed.md |
| Sector map on the shared view controls (V10a): flat chart spec, HUD strips labels and overlays and keeps the grid; whole hex takes a press; stage 5 walk start as an e2e | done (Traveller pack track); the grid as a pack overlay on other charts (gap 18), Layers names | packages/X3e-hud.md |
| V3 gap 25 part 1: a seat's brand (name and small raster logo) on its screens; join, waiting and away words from the packs' vocabulary (`vocabulary.join`), Traveller wording | done (Traveller pack track); part 2, quick overrides on the Seats panel and the size coming | packages/G25-brand-join.md |
| V3 gap 23 part 1: Describe on the body card hands Minerva one subject and its neighbours for one audience; a passed text kept as a narrative description note in the record (`session.addNote`) | done (Traveller pack track); the description on the card, and players seeing it through the record channel, next | packages/G23-describe.md |
| V3 gap 25 parts 1 and 2: a seat's brand (name and small raster logo) on its screens; join, waiting and away words from the packs' vocabulary (`vocabulary.join`), Traveller wording; quick overrides (`Seat.live`, Hide labels) from the Seats panel, live, never saved | done (Traveller pack track); Hide orbit lines, the single table and sealed seats, and the size coming, next | packages/G25-brand-join.md |
| V3 gap 18 part 1: the Traveller sector grid as pack data (`provides.other.overlays`, the hex map's own grid) on Voyages, placed on a star, under the overlay slot of PR 271 | done (Traveller pack track); imported worlds tied to the stars under their hexes next | packages/G18-sector-grid-overlay.md |
| Example packs 4.1: the Traveller standard ships (12 starships, 6 small craft) as templates and as pack ships the voyage planner flies and the designer draws (`nav_ships`, `packShips`); Mongoose's notice word for word (`provides.other.notice`, shown on Packs); the World hooks tag set (`sse.world-hooks`, 13 prompts) | done (Traveller pack track); pack ships in the voyage pickers (travel thread), tidal lock hooks | packages/X3g-standard-ships.md |
| A game system's difficulty numbers beside SSE4's words: `provides.other.difficulty` (dice, ladder, typical operator, words, skills), ui-contract `gameDifficulty` from odds or words; Traveller's ladder and skills from the Cepheus Engine SRD | done (Traveller pack track); shown on the At world card and route console (travel thread) next | packages/X3h-difficulty.md |
| Traveller sensors as instruments (RK11 pack share): each of the twelve kinds carries an sse-sense instrument (method, band, aperture, noise) so `detect` computes its reach; V3 ranges kept as authored | done; triggers asking `detect` (kernel lane), fits per tech level (FT8) next | packages/X3i-sensor-instruments.md |
| Resources on the books (TG11 kept half): `resource.<kind>.extracted` and `.stock` kept on any address, held = natural - extracted + stock in `read_inventory`, new op `transfer` that conserves | done; mining step and `resource.*` labels (X3k) next | packages/X3j-resource-books.md |

#### Open now

- **PR 400** (X3j, resources on the books). Green, and it changes `tests/golden/contract.jsonl`. It merges in the golden lane after the travel flight log (PR 403) and the kernel's barycentre PR. To land it: merge main, run `SSE_BLESS=1 cargo test -q -p sse-kernel contract_matches_golden`, check, push, merge on green.

#### Next packages, in order

1. **X3k: mining and resource labels** (TG11 program step, TG8 part 2).
   - A ship's program moves ore at a computed rate by `transfer`.
   - `resource.*` labels are derived from `held_kg`, so a mined-out rock loses its label by itself.
   - Packs name their making and consuming effects (a refinery).
   - Check: helium-3 on all four giants (MP8c: kind with matter `he3`, reach `envelope`).
2. **Sector routes as links.** Import the routes in sector files (trade, courier, jump lanes) through the travel thread's `set_link` (PR 377), with `source` set to the import and `audience` and `tier` for routes players don't know. `plan_voyage` already uses crew-known links.
3. **RK12, the Traveller mapping** (Atomic Rockets plan, accepted in full): weapons as "effect at range" (in range, marginal or out of range; nothing fired), with the game's table beside the physics. Then the RK16 pack side: stealth and countermeasure gear onto the counters list. Design questions go to the Atomic Rockets track.
4. **MP16 counted measures.** "2 jumps" and a "Jump-2" drive rating (never "J2"). The default count is ceil(route pc / rating), and the GM's count overrules it. This needs the kernel's counted-measure declarations, and links carry `counted` in `nav`.
5. **WBH extended profile** as one swappable "big block".
   - Read the physical profile off SSE4's physics; never roll a physical digit. Then add a seeded, GM-only social layer.
   - Use Cepheus Engine SRD (OGL) numbers for the basic social rolls, and keep WBH tables out (Mongoose copyright).
   - Port ideas, not code, from Roy Martin's TravellerSystemGenerator (GPL-3.0, so its code cannot enter this Apache repo). Credit it in sources and the log according to its licence, and say nothing about permission.
6. **Smaller items:**
   - Triggers that ask `detect` for the pair instead of reading `range_m` (kernel lane).
   - Fits per tech level, and the Traveller TL table as a tech ladder (FT8, after FT1 and FT2).
   - SV7: the pack names its own "Known to" rungs.
   - Switch `shipLog` to `crewAboard` (PR 318).
   - Gap 25 rest: hide orbit lines, sealed seats.
   - Gap 18 rest: imported worlds tied to the stars under their hexes.
   - Tidal-lock hooks once a lock fact exists.
   - OV10 pack half: allegiance and travel zones as hex map layers.

#### Alex's rulings for this track (23 and 24 Nov)

- Never the TAS logo or other trademarked art. Ship no copyrighted rules text, and bundle no official setting data.
- The Traveller pack data alone is CC BY-NC 4.0 (`packs/traveller/LICENCE.md`, which quotes Mongoose's fair use notice word for word); the repo stays Apache-2.0. Alex emails Mongoose (sales@mongoosepublishing.com) himself.
- Traveller is the 4.0 showcase. Other game packs (Mothership game side, Eclipse Phase, 2300AD) wait for 4.2.
- SSE4 never rolls dice or decides. A game pack may show its own difficulty ladder beside SSE4's words: odds through the system's dice first, then the word table; GM screens only; the skill shown when the pack knows the game's skills (X3h).
- Sensors must "just work", and stay tweakable by those who care (X3i).
- The World Builder's Handbook way of making worlds is wanted. It ships as one "big block" a GM can grab and change (see next package 5).

---

## `handover/status/manifest.md` from PR 423 (branch `claude/project-thread-mm0wul`)

### Status: ship manifest and crew

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

#### Alex's rulings for this track (so they live in the repo, not only in the thread)

- 23 Nov 19:53: "do the manifest and crew work... that IS good"; SSE3 had crew and life support as text boxes, SSE4's is a general tracking tool; "it will get a makeover with aliens [4.1] and again in RPG [4.2]". Keep it light and never assume crew are human.
- 24 Nov 14:00, **context by desk**: "explore/run will allow manifest/crew/fuel updates but not allow changes to the ship (and the pic & the 3d model we create or they supply)... the edit tab is the tool to hide things". So: in play (Explore, Run) what is aboard changes (people, cargo, roles, states, amounts, fuel); the ship itself (facts, fittings such as recycling, design, pictures, default view) changes only on Build. `ShipPlans` takes `locked` on play screens.
- 24 Nov 14:15, **picture selector**: "an image selector, pic, 2d 3d whatever the player loads... they can hold the default image as the last selected - which may be the 3d model". Built as the general `Pictures` panel (views/pictures); 14:28 he wants the same on the body card ("same as a ship"), which the chart room thread is doing.
- 24 Nov 14:18 to 14:19, **stations**: generated from their description, land constructs the same; about two dozen shapes sized to population with bolt-ons; docking points by size; a cut-off where it becomes a megastructure because it then touches physics. Built by the renderer thread (StationPlans, PR 408), hosted in the same Pictures panel.
- 24 Nov 14:20, **fuel**: "fuel gauge (and an edit fuel) to top up, etc - yup go for it!" Built (MC4, MC5); voyages planning from and drawing down the tanks is the travel thread's, agreed with it.

#### Open questions for Alex (each with the track's recommendation; built on the recommendation meanwhile)

1. The six crew questions of 23 Nov (aboard is one kind of thing for people and cargo; kinds of being never assumed human; own age that stops in cold sleep; life support from the stores aboard; damage only through readiness; players learn of entries through Known to): no answer yet; all built as recommended.
2. Should players see a ship's pictures once the ship is shown to them? Recommended yes, through the same Known to eye. Blocked on a player-side ship card, which does not exist yet (player screens show no construct card).

#### Next

- After PR 423 (MC5) merges: nothing queued that needs no one else. Waiting on others: the travel thread plans voyages from `fuelNow` and draws the tanks down with `setKept` (agreed 24 Nov); the chart room puts `Pictures` on the body card.
- Candidates, in order: a player-side ship card (with the chart room) so question 2 can be built; crew limits by kind of being with FT2 (future tech); `tests/ui/manifest.e2e.mjs` into CI's e2e list (CI edits need Alex's own yes).
