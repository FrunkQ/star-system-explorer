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
- `aboard` (3,314 legs): person, from_s and to_s (universe seconds; to_s null for those still up at the list's
  date), the craft or station in McDowell's words, `on` (the construct id of its log here, null for suborbital hops,
  Challenger's last flight, the Soyuz MS-10 abort and flights newer than GCAT's tables), role, ride, mark and src.
  A crew that goes up on one ship and down on another has both in its legs; a stay on a station is aboard the
  station; moonwalkers are aboard their lander. Pad aborts (RSLS) are left out, as they never left the ground.
- Short flights carry `peak_km` from McDowell's suborbital table (suborb.html, 41 flights to Nov 2024; newer ones
  null, meaning unknown), so the pack value `edge_of_space_km` (100, Alex's ruling on track SH question 11) can mark
  those between McDowell's 80 km (`listed_from_km`) and 100: 74 of the 133 legs with a height. Aborted X-15 drops
  and Soyuz T-10-1's pad escape are left out, as they never reached 80 km.
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

## Part 2 (25 Nov 2026): In memory

`scripts/spaceflight/in_memory.py`, called by build_crews.py, writes `store-packs/sol-today/in-memory.json` and puts a
`death` on each person it names. Losses: 6 on flights (Soyuz 1, X-15 3-65-97, Soyuz 11, Challenger, Columbia, VSS
Enterprise), 8 preparing to fly (Bondarenko, Freeman, See and Bassett, Apollo 1, Williams, Lawrence, Gagarin,
Vozovikov) and 3 in ground work (Nedelin 1960, toll 78 to 121 as the sources differ; Plesetsk 1980, 48; Alcântara
2003, 21). Every loss has McDowell's list or the Wikipedia list's row (read 25 Nov 2026) and the event's or each
person's Wikidata item; the script stops if a Wikidata date of death or toll disagrees. Those McDowell lists only
as trainees join crews.json as people with no legs (ids TR-...). Where the toll is larger than the names the
sources give, `unconfirmed` says so in words (Gagarin's instructor is not named in these sources). Sentences are
plain facts in our own words from the Wikipedia rows.

Also: Challenger's last-flight legs carry `reached_space: false` and are left out of `in_space`; `name` drops
McDowell's "*" marker (his form stays in `name_mcdowell`); pack.json gains the calendar the roll call writes dates
with (Gregorian, days counted on TT).

Waiting for a second source, so not in yet: Xichang 1996 (Long March 3B and Intelsat 708, 6 to 100), the STS-1
nitrogen deaths of 19 March 1981 (John Bjornstad, Forrest Cole, Nick Mullon in the Wikipedia row), the Scaled
Composites test explosion of 26 July 2007 (3), the R-9 silo fire at Baikonur of 24 October 1963 (7). A note on
Freeman: McDowell's trainee list gives 21 October 1964; Wikidata and the Wikipedia list give 31 October, which is
used.

The test `in_memory_names_only_what_two_sources_give` checks two sources and a sentence per loss, each named person's
death points back, no toll under the names given, and nobody in space as Challenger flew.

### Part 2 shapes, agreed with "Build roll call and In memory" (track SH, 25 Nov 01:23)

- A person's `death`: { at_s, date (the record's calendar date, text), where, `on` (log id when on a craft), during
  ("flight", "training", "ground_test", "ground_work"), loss (id), sources [at least 2 {text, url}] }. Training and
  ground deaths are people with no aboard legs.
- `store-packs/sol-today/in-memory.json`: `losses[]` { id, at_s, date, where, during, sentence (one plain sentence
  of fact), on, people [ids], unconfirmed (text or null), toll (sourced number, or [low, high]), sources [2+] };
  animals that died on flights too (with `being` on the person); `earth_memorials[]` (Space Mirror, Kremlin wall,
  Star City) as text with sources.
- A lost craft's log ends in the ordinary `gone` state with how "lost".
- `store-packs/sol-today/memorials.json`: `memorials[]` { id, kind ("object_left", "named_feature",
  "named_small_body", "remains_carried"), name, people, names_text, since_s, since_date, where { construct, body,
  lat_deg, lon_deg } or { mpc, elements {a_m, e, i_deg, node_deg, peri_deg, m0_deg, epoch_s, frame}, src },
  sources [2+] }. Real memorials only (Alex, 25 Nov: "stay restrained and respectful - one place we don't joke").

## Part 3 (begun 25 Nov): memorials

`scripts/spaceflight/memorials.py` writes `store-packs/sol-today/memorials.json` from
`scripts/spaceflight/memorials_sources.json` (`--fetch` queries again: one Wikidata query for asteroids whose "named
after" (P138) is a person in memory, with their Minor Planet Center number (P5736); then one JPL Horizons OBJ_DATA
record per number). 21 bodies: 1772 Gagarin, 1789 Dobrovolsky, 1790 Volkov, 1791 Patsayev, 1836 Komarov, 2161
Grissom, 3350 to 3356 for Challenger's seven, 8032 Michaeladams, 51823 to 51829 for Columbia's seven. Elements are
the Horizons osculating set at its solution epoch, heliocentric ecliptic J2000; `since_s` is null until the MPC
citations are read. Run it after build_crews.py (it reads the people's Wikidata ids from crews.json).

Held for a second source: 23 lunar craters and Columbia's five hills on Mars (Wikidata gives coordinates; the IAU
Gazetteer at planetarynames.wr.usgs.gov was unreachable from the build container), the Fallen Astronaut and its
plaque at Hadley, Clyde Tombaugh's ashes on New Horizons, Eugene Shoemaker's on Lunar Prospector.

The test `memorials_are_real_with_two_sources` checks two sources each, every person named is in memory, bound orbits
between 1 and 3.5 au, and 3350 Scobee at 2.311 au.
