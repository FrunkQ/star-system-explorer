# BX2: deaths in a pair as jumps (part 1, pairs)

Thread "Draw binary mass transfer and flares" (27 Nov 2026), track BX (`handover/status/binaries.md`). Alex's ruling 5: a star's death is a jump (instant mass loss and a kick, Blaauw 1961), not the a·M rule. Part 1 does this for two stars aged together (`sse-time/src/pair.rs`). Part 2 does it for a lone star's planets (`epoch.rs`). Part 2 moves the core golden, so it needs a lane turn.

## Kernel

New formula block `sse.orbit.kick_jump` (`formulas/orbit.json`). It is Brandt and Podsiadlowski 1995: the orbit just after one of two bodies loses mass at once and is kicked, found from the new relative speed at the same place. Its Rust copy is `sse_atoms::binary::kick_jump`, held to the block by `kick_jump_block_matches_its_sum`. That test also checks Blaauw's closed form absolutely: a' = a Ma/(2Ma − Mb), e = (Mb − Ma)/Ma; losing more than half unbinds the pair, and a backward kick can keep it bound. New kinds: `binary.total_mass`, `binary.kick_velocity`, `orbit.distance`, `orbit.velocity` (atom count 188).

`crates/sse-time/src/pair.rs`:

- **Ending the step at the collapse.** A star whose line ends in a neutron star or black hole ends its step at the moment of collapse. Its mass drops at once to the remnant's, and the ejecta count in `lost_msun`. Its age is clamped to its line's end from then (`collapsed`), so winds never run backwards.
- **The kick.** Three normal draws times 265 km/s for a neutron star (Hobbs et al. 2005). For a black hole, that times 1.4 over the remnant's mass. The draws come from the new `seed` argument, keyed by the star's address (stream `time.pair.kick.v1`).
- **Where on the orbit.** The point of collapse is drawn uniformly in time (mean anomaly), so eccentric orbits are handled. So is the direction of the line between the two in the plane.
- **The pair's new orbit.** The pair either jumps to a new orbit or comes apart. A `supernova` event gives the masses, the kick, and the orbit before and after (or the speed apart). Comes apart: an `unbound` event, and rows carry `apart: true`. Each star then ages alone.
- **Planets.** Each planet round the dying star feels the same loss and the same kick vector (with the opposite sign), at its own drawn place. It is kept on a new eccentric orbit (`eccentricity` on `PlanetFate`) or set loose (`fate: "unbound"`). Engulfment now uses the periastron.
- **Stripped massive donors.** A massive donor stripped of its envelope no longer becomes a white dwarf. It is a helium star of its core's mass (HPT00 eqs. 77 and 78; Wolf-Rayet phase) until its line's end, then it collapses. The "stripped" event says which.
- **The core of a star bound to collapse.** Past its main sequence, that core is the helium star as bright as it (HPT00 eq. 77 inverted). Before this, the sketch gave a 12 solar mass star a 0.9 solar mass core.
- **Kernel op.** `AgePair` takes an optional `seed` (default "sse4").

## What it does now

| Pair | Story |
|---|---|
| 10 + 9 at 500 au, no kick | At 26.9 Myr the 10 collapses to a 1.4 neutron star. The orbit jumps to 2889 au, e 0.83 (Blaauw exactly). Its planet at 20 au is set loose. At 32.7 Myr the 9 collapses and the orbit goes to 3897 au, e 0.36. |
| 10 + 9 at 500 au, seed "sse4" | A 520 km/s kick unbinds them at 521 km/s. |
| 12 + 8 at 1 au | Stable transfer at 17.9 Myr strips the 12 to a 5.1 helium star. It collapses at 19.7 Myr. The seed decides whether the pair holds; over 24 seeds, some hold on eccentric orbits and some part. |
| 40 + 30 at 2 au | The 40 is stripped to a 22 helium star and collapses to a 10 black hole (73 km/s kick). The pair holds at 8.7 au, e 0.71. The 48 solar mass accretor collapses at 7.3 Myr and unbinds them. |

## Checks

`cargo test -p sse-time pair_tests`, new tests:

- `a_death_without_a_kick_jumps_as_blaauw_said`: absolute Blaauw separation and eccentricity to 1e-9; 8.6 solar masses lost; the dying star's planet is unbound at the collapse; the partner's planet is untouched (20 au, e 0). Seen red with the jump replaced by the a·M rule: separation 913 au, e 0.
- `a_stripped_star_explodes_and_the_kick_decides`: stripped, then a supernova; a Wolf-Rayet star under 2 R☉ and over 30,000 K in between; the same seed gives the same line; across 24 seeds, both outcomes happen.

The BX1 and BX3 tests are unchanged and pass. Area `pairs` re-blessed; no core line moved.

## Not done

- Lone stars (part 2: `epoch.rs` deaths and their planets; core golden).
- A death inside a pair acting as one body one level up is still slow loss to the outer orbit.
- Kick spreads by channel (electron-capture and ultra-stripped supernovae kick less); the blast's push on the companion; planets out of the pair's plane.
- Helium star winds and evolution beyond the zero-age helium main sequence; a lone star's core is still the sketch.
- What the remnant does with gas it is given (BX4).
