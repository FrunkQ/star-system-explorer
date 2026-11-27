# BX1: two stars aged together

Thread "Draw binary mass transfer and flares" (27 Nov 2026), track BX (`handover/status/binaries.md`). Before this, F7's rule table (`crates/sse-time/src/binary.rs`) said when a star filled its Roche lobe and what kind of episode followed, but carried nothing forward: neither star's mass nor the orbit changed. BX1 ages both stars in one window with the masses they really have.

## Kernel

Three formula blocks, each with a Rust copy in `crates/sse-atoms/src/binary.rs` held to it by `pair_blocks_match_their_sums` (atom count 182 to 185):

- `sse.orbit.roche_lobe` (`packs/base-physics/formulas/orbit.json`): Eggleton 1983.
- `sse.orbit.overflow_rate` (same file): Hurley, Tout and Pols 2002 eq. 58, capped at 5 Msun a year.
- `sse.star.thermal_timescale` (`formulas/star.json`): the envelope's Kelvin-Helmholtz time (HTP02 eq. 61).

`crates/sse-time/src/pair.rs`:

- `Node` is a star or a pair of nodes (`{"node": "star", ...}` / `{"node": "pair", "a", "b", "separation_au", "eccentricity"}`). A pair of two stars is stepped as two stars; a pair holding a pair treats each side as one body one level up, its mass read from its own line (Alex's "pairs of paired mass centres").
- Each step reads both stars from their life lines at the mass they have (`AgeingStar::state_with_mass`), works out each lobe fill, and moves gas at the overflow rate, never faster than the donor's thermal rate. The accretor keeps at most 10 times its own thermal rate. The rest leaves from beside the accretor with its specific angular momentum, and the orbit is solved from the angular momentum that is left. Away from contact, winds widen the orbit as a·M stays fixed (Jeans 1924).
- At each new contact, F7's q_crit rule decides between stable transfer, a common envelope (Webbink 1984, `binary::common_envelope_separation`) or a merger. It is judged again at every later contact, so Algol's second episode (the grown accretor, now a giant, filling its lobe onto the white dwarf) is its own episode.
- A main-sequence star whose mass changes by more than 1 % is rebuilt at the new mass, keeping its share of the main sequence (an accretor's share falls as the mass ratio). A donor stripped to its core becomes a white dwarf of that core.
- Planets (`PlanetIn` with host and semi-major axis) are engulfed when a star's radius reaches them. Otherwise they widen with their host's mass loss (`PlanetFate`).
- `PairLine` gives dated rows for each side (mass, radius, luminosity, temperature, phase, lobe fill) and the orbit (separation, period, mass lost), plus events, planet fates, inner lines, why, stand-ins and references.

Op `age_pair {node, to_myr, z?, planets?}` answers `pair_aged {line}`. The star settings come from the stack's time settings (`no_time_settings` without them, as for `look_ahead`). It is in the `formation` perf chunk.

## Contract

New area `tests/contract/areas/pairs.json` → `tests/golden/areas/pairs.jsonl`: Algol 3 + 1.5 Msun at 0.08 au to 450 Myr; 1.0 + 0.3 at 1 au with a planet at 0.4 au; a 1.2 + 1.0 pair at 0.5 au with a 0.7 at 100 au; a negative age (error). The area loads `$pack:base-time`, a new frozen copy at `tests/fixtures/packs/base-time.json` (the contract runs on frozen packs). Area-only golden: no core line moves. `tests/golden/contract-reach.txt` re-blessed for the new op.

## Checks

`cargo test -p sse-atoms pair_blocks`; `cargo test -p sse-time pair_tests`:

- A wide pair keeps a·M fixed and its orbit widens past 1.5 times.
- Algol reverses its mass ratio (the accretor passes 2 Msun) and the orbit shrinks then widens, with mass conserved less what left.
- A 1.0 + 0.3 pair at 1 au shares an envelope and leaves a white dwarf with a period between 0.05 and 10 days.
- A 2.0 + 0.5 pair at 0.02 au merges.
- A planet at 0.4 au is engulfed, while one at 5 au is kept and widened past 6 au.
- A nested pair acts as mass centres.

`show_algol` (ignored) prints the whole Algol line. Run on, Algol's 4.17 Msun giant fills its lobe onto the white dwarf at 552 Myr: a common envelope, leaving a double white dwarf at 0.0013 au.

## Not done (later packages)

- Magnetic braking and gravitational waves (BX3), so a post-common-envelope pair never comes back into contact.
- Novae and X-ray binaries (BX4).
- Deaths as jumps with kicks (BX2).
- The stream and disc at a date (BX5).
- The rule table and `PairSettings` are code defaults until they join base-time (fudge register).
