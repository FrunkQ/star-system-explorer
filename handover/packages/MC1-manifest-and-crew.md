# MC1: ship manifest and crew roster

Track: ship manifest and crew (MC). Date 23 Nov 2026. Asked by Alex, 23 Nov: "do the manifest and crew work... that IS good"; "we have crew, life support, etc in SSE 3 - as text boxes - but this is a general useful tool for tracking this and future work"; "it will get a makeover with aliens and again in RPG". Design origin: example packs design section 4.3A (the general pieces), with sections 0 and 4.3A's game-specific parts still parked for 4.2.

## What a GM can do

- On **Ships and stations**, a ship's card has one new button beside Log: **Aboard** (laid out with the chart room thread, 23 Nov, so the card's row stays Aboard, Log, Power, Take out of play). It opens one panel with a two-way switch, **Crew** (first) and **Manifest**, the same for a ship, a station or a site, and it stays open as the card changes ship.
- **Crew**: everyone aboard, by name, with role (the setting's own words), kind of being, state (awake, in cold sleep, missing, dead) and own age now. **Add someone…** brings a person aboard; the state list changes them in place; **Move…** sends them to another ship, station, world or the system. Edit mode adds an inline role field.
- **Manifest**: people aboard against the ship's crew limit and berths; the ship's own capacities from its design (cargo, tanks); readiness; the share of oxygen and water recycled (edit mode); **Life support**: which store runs out first and when, each store's mass and days, the daily use at Science depth; anyone of a kind of being with no figures is named apart ("No life support figures for vesh"), never given a human's; stores in counts or volumes are named as not weighed. **Cargo and stores**: every lot with amount and unit, **Load…** and **Move…**, amounts editable in edit mode.
- Everything is a dated record change, so it undoes and replays like any other kept value, and the history keeps where someone was.

## How it works (no kernel change)

- Everyone and everything aboard is a record **actor** (record design 2, "actors that are not places") at the universe root, kind `person` or `cargo`, with kept values from the new base pack `sse.base-manifest`: `aboard.on` (ref: where it is), `aboard.role`, `aboard.state` (choice), `aboard.being`, `aboard.age_yr`, `aboard.what`, `aboard.amount`, `aboard.unit`, and `life_support.recycled` on the construct. Writers: the GM (programs and pack rules can be added as writers later; the 4.1 autopilots' `transfer_cargo` is one `aboard.on` change).
- Why actors at the root and not children of a ship: a person or a crate moves between ships, and an address must not change when it does. Where it is, is a relation (record design: relations are kept refs).
- Life support (`ui-contract/manifest.ts` `lifeSupport`): each store's mass over the net daily use of everyone living aboard at their kind of being's rates (`provides.other.life_support.rates`, human from NASA's BVAD, unverified), less the recycled share for stores the rates mark recyclable. Stores are matched by `aboard.what` through the pack's `store_words` (rations are food).
- Age (`ageNow`): the age last set plus campaign time awake since; cold sleep and death stop it. Both stand-ins are in the fudge register.
- The pack is on the base stack (`packages/shell/src/documents/stack.json`), so every setting has it.
- Session: `addActor` (kernel `add_actor`) and `setKept` (kernel `set_kept`), GM only.

## Checks

- `cargo test -p sse-record`: `the_manifest_pack_moves_people_between_ships` (the pack's kinds load; a move is one dated change with its reason; the earlier place reads back at an earlier time; an unlisted state or writer is refused).
- `tests/ui/manifest.test.ts` (8): pack kinds and cited rates, reading entries, who is aboard where, ageing with cold sleep and death, life support days and recycling, unrated kinds named apart, unweighed stores, new ids and starting values.
- `tests/ui/manifest.e2e.mjs` (chromium, built shell; run it by hand: `node tests/ui/manifest.e2e.mjs`. It is not yet in CI's e2e list, which needs Alex's yes to change): Traveller pack added, two ships made, three people aboard (one vesh), stores loaded; oxygen first at 37 days for two awake, 73 days with one put in cold sleep; machine parts moved to the other ship; phone width fits. Screenshots: `/mnt/project-files/sse4/handover-screens/MC1-screens/`.
- No golden hash changes.

## Rough edges and next

- **Players.** Entries are record actors, not in any player scene, so they are GM only for now. SV1 (kernel thread, landing after its queued PRs) covers actors: once `set_known` puts an actor at heard of or known for a crew, it reaches that crew's scene, and the `aboard.*` values show because their tier is `scanned`. Next here: the eye on each entry and the card title (SV2's component), once SV1 is on main.
- **The ship's log** reads the construct's own record; moves of people and cargo are on their own addresses, so "Ada came aboard" is not in the log yet. Next: `shipLog` takes the aboard entries' `aboard.on` changes that name the ship.
- **People aboard, one number** (agreed with the Traveller pack thread, 23 Nov): `crewAboard(subject, roster)` in `ui-contract/manifest.ts` is the count every screen reads: the roster's people aboard (not dead or missing) when the ship has a roster, else the template's starting figure `construct.crew`. The templates' `traveller.crew` and `traveller.berths` are the ship's design figures (complement and berths), not people aboard. The ship's log and the "At world" card should read `crewAboard` too.
- **Damage to parts** waits for parts on ship records (ship designer). Readiness and the damaged status are shown.
- **Crew limits from the kind of being** (future tech limits 4, 5 and 9: acceleration, weight, air and food) join with FT2 on civilisations.
- 4.1 (aliens) brings rates per kind of being; 4.2 (RPG) brings character sheets from pack data, which will hang off the same `person` actors.
