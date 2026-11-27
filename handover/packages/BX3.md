# BX3: feeding after a common envelope

Thread "Draw binary mass transfer and flares" (27 Nov 2026), track BX (`handover/status/binaries.md`). Built on BX1 (`packages/BX1.md`). In BX1 a pair could only come into touch by a star swelling. A white dwarf left days from a red dwarf after a common envelope therefore stayed that way forever. Real pairs lose orbital angular momentum to gravitational waves and to magnetic braking. That loss draws them into touch: a cataclysmic variable.

## Kernel

Two formula blocks, each with a Rust copy in `crates/sse-atoms/src/binary.rs` held to it by `pair_blocks_match_their_sums` (atom count 185 to 187):

- `sse.orbit.gw_decay` (`formulas/orbit.json`): Peters 1964. Check: the Hulse-Taylor pair would shrink 0.3 m a year if circular. The measured 3.5 m comes from its eccentricity, which this block leaves out.
- `sse.star.magnetic_braking` (`formulas/star.json`): Hurley, Tout and Pols 2002 eq. 50.

`crates/sse-time/src/pair.rs`:

- Each step, the orbit loses angular momentum to waves, and to braking from any main-sequence star between 0.35 and 1.25 solar masses with its spin locked to the orbit (period under 10 days). The convective envelope share is HPT00's M_env,0. A star convective through (below 0.35) brakes no more, which is the period gap's cause.
- A main-sequence donor onto a compact star has its own critical ratio (1.0, Claeys et al. 2014). Above it, the contact goes through the common-envelope branch, not the two-dwarfs merger.
- A red or brown dwarf feeding a compact star gives just what keeps it at its lobe. This is solved each step by bisection: the gas lost from beside the accretor widens the orbit, the dwarf shrinks as M^0.8, and a brown dwarf grows as M^−1/3. The amount is capped at 100 times the dwarf's thermal rate. Brown dwarfs can now fill their lobes.
- The contact event says why it happened: "magnetic braking and gravitational waves drew the orbit in from ... to ...". It is named `cataclysmic-variable` when a main-sequence dwarf feeds a white dwarf, with the period in hours.
- Step control: no step moves the orbit by more than the radius step.

## What a 2 + 0.4 solar mass pair 1 au apart now does

| Age (Myr) | Stage | What happens |
|---|---|---|
| 1292 | common envelope | Leaves a 0.45 white dwarf 0.0087 au from the red dwarf (P 7.7 h). |
| 1353 | contact | Braking draws them into touch at P 3.0 h: a cataclysmic variable. |
| 1353 to 1357 | above the gap | Fed at 1e-8 to 3e-8 solar masses a year, down to 0.33. |
| 1357 onward | below the gap | Braking is off. Waves alone feed it at about 3e-11 to 4e-11. |
| about 8900 | period minimum | P falls to 63 min at the brown-dwarf boundary. |
| to 13700 | bounce | The brown dwarf, now 0.03, is fed at about 3e-12 while the period rises to 95 min. |

## Checks

`cargo test -p sse-time pair_tests`, new tests:

- `braking_and_waves_make_a_cataclysmic_variable`:
  - contact between 2 and 8 hours;
  - a peak rate above the gap between 1e-9 and 1e-7;
  - a middle rate below it between 1e-11 and 1e-9, all under 1e-9;
  - a period minimum between 0.03 and 0.07 days, then a rise past 1.2 times it;
  - a white dwarf and a brown dwarf at the end.

  Seen red with braking off: contact happens, but the top rate is 7e-11.
- `white_dwarf_pairs_spiral_in`: Algol's double white dwarf shrinks. Seen red with waves off: the separation stays at 0.0013 au.

The BX1 tests are unchanged and pass. Area `pairs` re-blessed; no core line moved.

## Not done

- Donors out of thermal balance, so the dwarf does not detach at the top of the gap (3 h) and come back at 2 h.
- The brown dwarf's radius is a fixed 0.1 R☉, which puts the minimum at 63 min, not the observed 80.
- Eccentric-orbit enhancement of the wave loss.
- Novae and what the white dwarf does with the gas (BX4).
