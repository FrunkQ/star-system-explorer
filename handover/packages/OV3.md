# OV3: holdings on map layers (kernel)

Track OV (`design/SSE4-territory-overlays-design.md`, sections 4, 5 and 8). Built 23 Nov 2026 by the kernel thread. The result shape was agreed with the renderer thread's OV2 `Holding` type, adding `until_s` and `source`.

## The record (`sse-record/src/holdings.rs`)
- **What a holding is.** It is one `record.hold` event about a place. The event names:
  - the layer and the answer (`key`: an actor's address, or any short key);
  - its status: `holds`, `claims` or `influences`;
  - optionally a `strength` from 0 to 1, an `extent`, who asserts it (`by`) and its `source`;
  - its tier, audience, writer and reason.
- **Where it is kept.** It lives in the event's data only, never in the fold's facts, so no flow can read it.
- **Slots.** A later event in the same slot replaces the earlier one, and `withdraw` ends it. There are three kinds of slot:
  - holds: one holder per place;
  - claims: one per claimant;
  - influences: one per answer.
- **`history`** gives every holding with `since_s` and, on those that ended, `until_s`. **`holdings_at`** gives those in force at a date.
- **Extents.** An extent is one of:
  - `{reach: {value, unit}}`;
  - `{band: [q, q]}`;
  - `{outline: [[x, y]...], frame}`, in one of the frames galaxy-plane-pc, cluster-pc, system-au, body-latlon or web-mpc.
- **Reading rules.**
  - `who_holds` applies, in this order:
    - **inherit down**: the deepest held place that contains this one, so a holding on a system answers for its moons;
    - **a holding's own extent**, then **the layer's reach**: nearest wins, ties go by address, so borders between stars run midway and anything beyond every reach stays bare;
    - **roll up**: a place with held parts takes the answer with the most weight. Each part's weight is its kept `weigh_by` number less what its own held parts take, or it counts as one. The place reads as `divided` (shares, largest first) below 0.6.
  - `who_holds_point` answers a point drawn on a place: an outline drawn there first, then the place's own answer.
  - `holdings_in` gives the answers for the bounds and for every place seen inside it.
  - Every answer carries `via` (held, inherited, rolled-up or rule), `from`, `also` (the claims and influences there) and `why`.
- **Whose map (`View`).**
  - The GM sees everything.
  - Another audience sees a holding only if its audience lists them and it is untiered or `obvious`. Data views refine this in OV9.
  - `according_to` draws one power's claims as if held.

## The kernel (`sse-kernel/src/territory.rs`, ops)
- **Layers.**
  - `provides.other.overlays[]` entries with `areas` are read at load.
  - The kernel reads `fill.between_stars.reach` (it must be a length) and `weigh_by` (a kept number's id, or `count`).
  - A GM's own `misc.*` layer needs no entry. Any other undeclared layer is refused (`unknown_layer`).
  - Only `by: {relation: "holds"}` is read so far. Tag groups, choices, facts, knowledge and reach sources come with OV10.
- **Ops.**
  - `hold {place, layer, key, status, strength?, extent?, by?, tier?, audience?, writer?, reason?, source?, withdraw?, at_s}` returns `held`.
  - `who_holds {place, layer, point?: {frame, xy}, at_s?, audience?, according_to?}` returns `who_held {answer | null}`.
  - `holdings_in {bounds, layer, at_s?, audience?, according_to?, history?}` returns `holdings_listed {holdings, changes?}`.
- **Positions.**
  - Drawn star systems (`sys:f…`), pinned ones included, are placed where they are at the asked date, in galactic coordinates.
  - The date is the galaxy's now plus the time since the campaign clock's start.
  - Places without a drawn system (an adopted `sys:0`) use address rules only.
- **Weights.** Weights are the record's kept numbers.
- **Physics wall.** `world.who_holds` joins `GAME_READERS`. A physics flow naming it is refused at load. The reader block itself comes with OV12.

## Checks
- `sse-record` holdings tests, 9 of them:
  - moons inherit;
  - slots and dates;
  - a five-region world rolls up divided, and so does its system;
  - weighing by population;
  - borders run midway between stars, and the void stays bare;
  - an outline answers points;
  - whose map;
  - the same answers in any order of writing;
  - the fold's facts are untouched.
- `sse-kernel` territory tests, 3 of them:
  - through the ops on drawn systems of the Sun's cell, including the reach rule and a second kernel writing in the other order;
  - undeclared layers and bad values;
  - a secret holding never reaches a crew answer.
- The tags test now also refuses a flow that reads `world.who_holds`.

## Golden
- Physics goldens are untouched.
- `tests/golden/contract.jsonl` has one line re-blessed: the "unknown op" error, whose list of ops now names `hold`, `who_holds` and `holdings_in`.

## Next
- OV10 (claim rules and candidates) at stage 5.
- OV12 readers (conditions, finder, sifters, naming's Local layer).
- OV14 has moved to 4.1 (Alex, 23 Nov).
