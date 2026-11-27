# Status: star pairs that live together (track BX)

Owner thread: "Draw binary mass transfer and flares" (26 Nov 2026). Covers: BX, star pairs and nested multiples aged together, mass passing between them, deaths in a pair, planets in pairs, and drawing the stream, disc and novae. SG6 (prominences) is recorded in `status/star-look.md`.

Only the owner thread edits this file. Logs are in `handover/packages/`.

## What was there (main 152f4d3, 26 Nov)

- F7 `crates/sse-time/src/binary.rs`: when a star fills its Roche lobe, a rule table (Hurley, Tout and Pols 2002; Webbink 1984) decides stable transfer, common envelope or merger. It writes a story line only: neither star's mass nor the orbit is carried forward. It has no transfer rate, stream, disc, nova or Type Ia.
- Ageing (`epoch.rs`) runs the primary and its planets only. A neutron star or black hole's mass loss is treated as a slow wind (the a·M rule), so its planets stay bound. In a real supernova the loss is instant and a kick is drawn.
- Formation: a companion cuts the primary's disc (Artymowicz and Lubow 1994). A pair closer than 2 AU (a fixed line) gets one disc round both. The companion gets no disc or planets.
- `sse-formation/src/hierarchy.rs` already builds every system as nested pairs; `stars.rs` checks each level with Mardling and Aarseth.

## Alex's rulings (26 Nov 2026, 19:23 to 19:37 UTC, thread above)

1. A star that will likely go nova should be seen being fed; draw what the physics gives.
2. Planets in pairs: run formation once round each star (each disc cut by the other) and once round a close pair. Combine the planets (don't average them), then keep what the stable-orbit edge allows (Holman and Wiegert 1999). Say in the app that it is an approximation, and log it in the fudge register. Use research tables only if the results look wrong.
3. Any number of stars as nested pairs of mass centres: a node is a star or a pair of nodes, each centre of mass acting as one body one level up. The centre is computed, never stored.
4. The stable edge trims everything a star holds: planets, belts, scattered disc and comet cloud. A cloud round the whole pair is allowed outside about three times the separation, out to the galaxy's tidal edge.
5. End stages: a star's death is a jump (instant mass loss and a kick, Blaauw 1961), for a lone star's planets too.

Coordinator, from the time scales audit (PR 651): age the pair in one window kept as lines, never in two legs; a supernova is a jump, not the a·M rule; write back all six orbit elements with their date; no new absolute time in a double, no new `_yr` clock field.

## Packages

| Package | What | State |
|---|---|---|
| BX1 the pair aged together | One recursive node-pair stepping in one window: both stars from their life lines with the masses they really have; Roche fill each step (Eggleton 1983); transfer at a rate (thermal or nuclear timescale of the donor, or angular momentum loss by magnetic braking and gravitational waves once the donor is small), taken out of the donor's budget and given to the accretor (with the share lost from the pair), the orbit following; common envelope and merger as now; engulfment of planets by either star; dated lines | built (log `packages/BX1.md`); magnetic braking and gravitational waves moved to BX3 |
| BX2 deaths as jumps | Instant mass loss and a kick drawn from the measured spread (Hobbs et al. 2005 for neutron stars, reduced for black holes by fallback): bound or not, the new orbit, and what each planet and the companion end up circling; for lone stars too | after BX1 |
| BX3 feeding after a common envelope | The pair's orbit shrinks by magnetic braking and gravitational waves until the companion fills its lobe on the white dwarf: a cataclysmic variable, with the transfer rate from that loss | after BX1 |
| BX4 what the accretor does | A white dwarf: novae when the hydrogen layer reaches its ignition mass (recurrence from mass and rate), mass gained or lost each cycle; a Type Ia when it reaches the Chandrasekhar limit. A neutron star or black hole: an X-ray binary | after BX3 |
| BX5 the flow at a date | An op giving, for any pair at any date: lobe fill, L1 point, the stream's path (Lubow and Shu 1975), circularisation radius, disc radii and temperatures (sharing the black hole disc light of PR 618 where it fits), the hot spot, the rate | after BX1 |
| BX6 planets in pairs | Ruling 2: one formation run per star plus one round a close pair, combined, trimmed by the stable edge; ruling 4 for belts and clouds | after BX1 |
| BX7 the pair life line | One dated line per pair for the star gallery's "One star's life" view: each star's phase, mass, radius and temperature; the orbit's separation, eccentricity and lobe fill; the episodes; where each planet ends up | after BX1 to BX4 |
| BX8 drawing | In True, on the sim clock: the donor filling its lobe as a teardrop (with VP-S2), the stream from L1, the disc and hot spot, nova outbursts | after BX5 |

## Next
- PR 658 (SG6 part 1), then BX1 (area-only, its own PR). SG7 (one set of active regions) waits for its core-lane turn.
- Then BX2 or BX3.

## Left on Alex's PC (for later cleanup)
- `C:\Development\sse4-sg6`, `C:\Development\sse4-main-check`, `C:\Development\sse4-sg6-capture` (Playwright capture scripts), worktrees and copies used for the SG6 capture.
- `wasm-bindgen-cli` 0.2.130 installed with cargo; a static server on port 9400 may still be running.
