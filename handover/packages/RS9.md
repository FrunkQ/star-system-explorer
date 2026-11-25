# RS9: everyone who has been to space

Track RS (real spaceflight, the Sol today store pack). Asked by Alex, 25 Nov 2026 01:00: "does this mean you can go
to any day in history and a crew roster of every who was in space on that day?" and 01:03: "yes - it makes this a
cool history tool", with a respectful way to honour those who died; the view and the memorial are designed in track
SH (thread "Design spaceflight history view and memorial", note `handover/design/SSE4-spaceflight-history-design.md`).

## Part 1 (25 Nov 2026): the roster from McDowell's lists

`scripts/spaceflight/build_crews.py` reads Jonathan McDowell's human spaceflight lists (astro.tsv: one row per space
traveller above 80 km; segments.tsv: one row per leg of each person's flight) and writes
`store-packs/sol-today/crews.json`:

- `people` (792): McDowell's id, name, short name, kind of being (human; Ham and Enos are chimpanzees, in the
  segments but not in astro.tsv), citizenship, born, died, and `died_in_flight` when the death falls within a day of
  the end of one of their flights (19: Soyuz 1, X-15 flight 3-65-97, Soyuz 11, Challenger, Columbia).
- `aboard` (3,322 legs): person, from_s and to_s (universe seconds; to_s null for those still up at the list's
  date), the craft or station in McDowell's words, `on` (the construct id of its log here, null for suborbital hops,
  Challenger's last flight, the Soyuz MS-10 abort and flights newer than GCAT's tables), role, ride, mark and src.
  A crew that goes up on one ship and down on another has both in its legs; a stay on a station is aboard the
  station; moonwalkers are aboard their lander. Pad aborts (RSLS) are left out, as they never left the ground.
- `in_space`: the count of people in space at each change, for the time ruler (peak 20).

Craft names resolve against the logs by name, with the few showcase names aliased (ISS, Skylab, Apollo 11, STS-1,
LM 5), Dragon crews and Shuttle flights by their bracketed flight name, and the lunar modules by number.

### Checks

`crews_ride_the_craft_their_logs_fly` in `crates/sse-nav/tests/sol_today_logs.rs`: ids unique, every leg names a
person, every `on` has a log, at most 10 legs fall outside their craft's logged flight (5 do: GCAT's and McDowell's
dates differ there), Armstrong and Aldrin are in Eagle and Collins in Columbia at 20:17 UTC on 20 July 1969, the step
count agrees with the legs and peaks at 20, and 19 died in flight. No golden change.

### Still to do (part 2, shaped by track SH)

- Training and ground deaths (Apollo 1, Bondarenko, the T-38 losses, Gagarin; Nedelin 1960, Plesetsk 1980,
  Alcantara 2003) from a second source, each with where, during, one sentence and two sources.
- Wikidata ids on people; Laika and the other animals GCAT names (McDowell's list holds primates only).
- Memorials as dated cargo (the Fallen Astronaut at Hadley, Tombaugh on New Horizons, Shoemaker on Lunar
  Prospector, Celestis flights) and named features and small bodies, with their sources.
- The legs as `aboard.on` changes on manifest `person` actors, agreed with the manifest thread.
- The edge of space as one pack value (100 km), with 80 km flights marked.
