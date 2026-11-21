# Y7a: star cells through the kernel; a promoted star is stored whole

Night sky design (`design/SSE4-night-sky-design.md`) section 11, package
Y7a. It must land before Y1 (the count tree), because Y1 changes which fill
stars are drawn. Built 21 Nov 2026 by the kernel thread.

## Finding before building
G5's stars (`sse-cosmos/src/stars.rs`: `cell`, `promote`) were not wired
into the kernel. No op drew a star cell and no save held a promoted star.
The harness `stars` engine runs only the fixed G5 golden, and no UI package
keeps a picked star (checked with the night sky design thread). So no save
needs migrating today. The loader rule below covers every future save.

## What it delivers
- **`StarParams.key_version`** (default 1; not serialised at 1). The draw streams are `cosmos.stars.count.v<n>` and `cosmos.stars.v<n>`. Version 1 is today's draw exactly (the G5 golden is unchanged). Raising it stands in for any change to the fill rules.
- **`Stars::draw_one(address)`**: the fill system at `<galaxy>/cell:<x>_<y>_<z>/sys:f<frame>k<slot>`, as today's rules draw it.
- **Op `star_cell {cell?, t_myr?}`** (read only): one 10 pc cell of the campaign's galaxy. The default time is the campaign's now (the galaxy's now plus the campaign clock's elapsed time). The default cell is the one at the Sun's radius. The reply lists every system with its address, whether it is pinned, where it was drawn (cell, frame, slot), mass, companions, birth, [Fe/H], position and star state. The campaign's galaxy is the median Milky-Way-like galaxy until G7, as for `set_system_age`.
- **Op `promote_star {address}`** (GM machine only): draws the star once and writes a `star.pin` event. The event's `star.description` holds the whole `StarSystem` as JSON (exact: serde_json's float round trip), with searchable copies of mass, birth, [Fe/H] and companion count. Promoting a pinned star again writes nothing and returns the stored description.
- **Loader rule**: `cosmos::star_pins` reads every stored description back verbatim. A pin is never redrawn: it claims its slot in its own cell and frame, or replaces the last fill system of a cell it has drifted into (G5's rule).

## Verified by
- `a_pin_survives_a_change_to_the_fill_rules` (sse-cosmos): `draw_one` returns the cell's own system. With key version 2 the fill is redrawn and the same address draws something else, but the stored pin is in the cell once, unchanged, and its state at a later time is identical to the bit.
- `a_promoted_star_is_stored_whole_and_survives_new_fill_rules` (kernel):
  - star_cell, promote, then star_cell again gives the same systems and numbers, with one pinned;
  - a second promote writes nothing (state hash unchanged);
  - an exported save imported into a fresh kernel gives the same cell reply;
  - with key version 2 the pin reads bit-identical while the fill changes;
  - a bad address is refused.
- The G5 golden (`pack_and_golden`) is unchanged.

## Goldens
`tests/golden/contract.jsonl` changed on purpose. The hello op list and the bad_request variant list gained `star_cell` and `promote_star`. Three cases were appended: a star cell, a promotion, and a refused address. No other golden changed.

## Not done
- Y1's count tree, which re-blesses the G5 golden, as ruled.
- Undo of a promotion: Y7 adds release ("a released, unvisited star returns to the cloud").
- The chart room has no screen for star cells yet; Y2's `sky_from_here` is the first consumer.
