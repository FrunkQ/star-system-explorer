# Status: Co-orbital bodies, Trojans for any world (track CO)

Owner thread: "Add co-orbital asteroids as general physics" (opened 26 Nov 2026). Brief: Alex, 26 Nov 19:13 UTC, after Anton Petrov's video on the Rubin Observatory's Neptune L5 Trojans and its papers: "I imagine many/most gas giants will have this - i.e. this is a physics fix - not a solar system fix?"

Only the owner thread edits this file. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

## The method (one way, no second)
Every world, any star: `crates/sse-formation/src/coorbital.rs` finds the inputs and runs the four `sse.coorbital.*` blocks in `packs/base-physics/formulas/coorbital.json` (written by `scripts/build-coorbital-blocks.py`, every number cited there; fudge rows CO-* in `docs/fudges.md`).
1. **Zone** (`trojan_zone`): L4 and L5 hold while 27 mu (1 - mu) < 1 (about 1/26 of the star's mass); the swarm is (8 mu / 3)^(1/2) wide in orbit size.
2. **Catch** (`capture_share`): when a giant stirs a belt, every world whose points hold catches k times its width, thinned by h past each giant between it and the belt, out of the stirred mass (conserved: it comes out of the belt's books).
3. **Neighbours** (`neighbour_disturbs`): a swarm is lost if an adjacent world at least as heavy is within 3% of a resonance of order 3 or less.
4. **Keep** (`long_lived_share`): when the system settles, a world keeps min(1, c mu) of its catch, none if disturbed; the rest is thrown out (`solid_ejected`). One swarm per world, one `TrojansThinned` event each.

On the Solar System (round figures, `coorbital_tests.rs`): Jupiter, Neptune, Earth and Mars keep; Saturn (Jupiter, 5:2), Uranus (Neptune, 2:1), Venus and Mercury lose. Jupiter keeps 7e-7 of the disc (Nesvorny et al. 2013), Neptune 6.5 times that (Schwamb et al. 2026: 4 to 10).

| Package | State | Log |
|---|---|---|
| CO1 Trojans for any world: the four blocks, capture by every world in every stirring, survival when the system settles, swarm width from the zone | done (this PR) | packages/CO1.md |
| CO2 temporary Trojans: worlds without a primordial swarm catch passing small bodies for a while (Earth's 2010 TK7 and 2020 XL5, Venus's 2013 ND15, Saturn's 2019 UO14, Uranus's two, Mars's 2023 FW14); steady state from the passing flux times the temporary lifetime | next (needs a present-day flux of near-planet small bodies) | |
| CO3 swarms born in place: a growing planet traps local planetesimals at L4 and L5 (Marzari and Scholl 1998, per Emery et al. 2024); kept only where no later migration or disturbance empties them (terrestrial planets like Mars) | queued | |
| CO4 survival against age (a young system shows more than an old one), and per-swarm L4 and L5 shares from the catch instead of the fixed 0.6 (`leftovers::L4_SHARE`, Jupiter's measured ratio) | queued | |
| CO5 the real Trojans (16,684 Jupiter, about 35 Neptune, Mars's 14, Earth's 2, Uranus's 2, Saturn's 1) as catalogue entries claiming these swarms' slots | waits on RW3 (catalogue format, PR 643) and RW's small-body import | |
