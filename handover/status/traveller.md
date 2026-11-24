# Status: Traveller pack

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
| Resources on the books (TG11 kept half): `resource.<kind>.extracted` and `.stock` kept on any address, held = natural - extracted + stock in `read_inventory`, new op `transfer` that conserves | done | packages/X3j-resource-books.md |
| Mining and resource labels (TG11 mining step, TG8 part 2): `resource.*` labels read off held kg, so a mined-out rock loses its label; base pack `sse.base-resources` (metal, water, hydrogen, helium-3); new op `mine` at a ship's `mining.rate_kg_h` | done; a refinery's named effects, a computed drill rate next | packages/X3k-mining-labels.md |
| X3l, sector routes as links: `sse-sector-io` reads the metadata's `<Routes>`; `import_sector` takes `metadata`, `route_audience`, `route_tier` and writes each route between two imported worlds as a link (source `import_sector`, GM only by default); the Sector map fetches or takes the XML and says which routes were left out | done; cross-sector routes, routes drawn on the hex map next | packages/X3l-sector-routes.md |
| RK12, the Traveller mapping: nine ship weapons in `provides.other.weapons`, each with the game's tech level, optimum band and damage (Cepheus Engine SRD) beside an sse-sense weapon; a standard target; fudge F54 | done; rings on the chart (renderer, chart room), RK16 pack side next | packages/RK12-traveller-weapons.md |
| WBH part 1, the extended world profile: `sse-codes` `value` segments (a fact, scaled, over a sheet constant, to significant figures); code sheet `traveller.extended` (twelve physical lines, none rolled) as the swappable block; a "Read as" sheet picker on World codes; credits for the Cepheus SRD and TravellerSystemGenerator | done | packages/WBH1-extended-profile.md |
| WBH part 2, the social codes rolled: `sse-codes::roll` (a pack's dice procedure as data, seeded per world and step, hand-set codes kept); kernel op `roll_profile` (preview, or authored codes with how each was rolled); Traveller `traveller.social` from the Cepheus SRD (government, law, starport, tech level, bases); a roll button on World codes | done; a whole sector in one ask next | packages/WBH2-social-rolls.md |

## Open now

- **PR 400** (X3j, X3k, X3l, the RK12 Traveller mapping and WBH parts 1 and 2 together). It appends three lines to `tests/golden/contract.jsonl` (the `transfer`, `mine`
  and `roll_profile` refusals) and merges in its turn in the golden lane. To land it: merge main, run
  `SSE_BLESS=1 cargo test -q -p sse-kernel contract`, `scripts/check.sh`, push, merge on green.

## Next packages, in order

1. **RK16 pack side:** stealth and countermeasure gear onto the counters list. Design questions go to the Atomic Rockets track.
2. **MP16 counted measures.** "2 jumps" and a "Jump-2" drive rating (never "J2"). The default count is ceil(route pc / rating), and the GM's count overrules it. This needs the kernel's counted-measure declarations, and links carry `counted` in `nav` (imported routes could carry it too).
3. **WBH next:** roll a whole sector's unset worlds in one ask; more social lines once there are OGL or our own numbers.
4. **Resources, next steps:** a refinery's named making and consuming effects; a computed drill rate; catalogue Sol compositions (Mercury reads 23% iron like every catalogue body; inventory thread).
5. **Smaller items:**
   - Triggers that ask `detect` for the pair instead of reading `range_m` (kernel lane).
   - Fits per tech level, and the Traveller TL table as a tech ladder (FT8, after FT1 and FT2).
   - SV7: the pack names its own "Known to" rungs.
   - Switch `shipLog` to `crewAboard` (PR 318).
   - Gap 25 rest: hide orbit lines, sealed seats.
   - Gap 18 rest: imported worlds tied to the stars under their hexes.
   - Tidal-lock hooks once a lock fact exists.
   - OV10 pack half: allegiance and travel zones as hex map layers; routes drawn on the hex map.

## Alex's rulings for this track (23 and 24 Nov)

- Never the TAS logo or other trademarked art. Ship no copyrighted rules text, and bundle no official setting data.
- The Traveller pack data alone is CC BY-NC 4.0 (`packs/traveller/LICENCE.md`, which quotes Mongoose's fair use notice word for word); the repo stays Apache-2.0. Alex emails Mongoose (sales@mongoosepublishing.com) himself.
- Traveller is the 4.0 showcase. Other game packs (Mothership game side, Eclipse Phase, 2300AD) wait for 4.2.
- SSE4 never rolls dice or decides. A game pack may show its own difficulty ladder beside SSE4's words: odds through the system's dice first, then the word table; GM screens only; the skill shown when the pack knows the game's skills (X3h).
- Sensors must "just work", and stay tweakable by those who care (X3i).
- The World Builder's Handbook way of making worlds is wanted. It ships as one "big block" a GM can grab and change (see next package 5).
