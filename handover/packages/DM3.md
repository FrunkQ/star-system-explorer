# DM3: the matter ledger of a web cell

Track GH (`design/SSE4-galaxy-histories-design.md`, section 4.1), built 23 Nov 2026 by the kernel thread.

## What it does (`crates/sse-cosmos/src/ledger.rs`, op `matter_budget`, fudge F48)
- One halo cell's matter today, from the web's density, split into
  - dark matter in catalogued haloes (G2), in haloes below the catalogue floor (the same conditional mass function, down to `floor_low_msun`, 10⁶ M☉ by default) and never collapsed into those;
  - ordinary matter as stars, cold gas, hot halo gas, thrown-out gas, black holes, and gas between galaxies that never fell in.
- Reservoirs come from G3 run once per half dex of halo mass on a median halo (`Reservoirs`), read off by each halo's mass. Hot halo gas is a halo's share of ordinary matter less what its galaxy holds and threw out, so every halo closes; gas between galaxies is the rest of the cell's ordinary matter, so the cell closes. Haloes below 10⁹ M☉ (about the filtering mass after reionisation) hold no galaxy, so their ordinary matter counts as between galaxies.
- The cell's volume and matter by web environment (voids, sheets, filaments, knots), on 8³ samples.
- G3's `Galaxy` gains `ejected_now` (thrown-out gas now), not on the wire.
- Settings: `provides.other.cosmology.ledger` (`LedgerParams`).
- Op `matter_budget {cell?}` (default the home cell), in the "Galaxy and cosmos" perf chunk; one contract line added.

## Checks (`crates/sse-cosmos/src/dm3_tests.rs`, and the kernel test `matter_budget_closes_and_is_repeatable`)
- Closure in 27 cells: the parts sum to the cell's matter, and ordinary over all matter is f_b, to 10⁻⁹.
- The census of ordinary matter over the 27 cells: stars 3.7 %, cold gas 1.3 %, hot halo gas 29 %, thrown out 11 %, between galaxies 55 %, black holes 6 × 10⁻⁵. Stars are under a tenth and thin gas over four fifths (Fukugita and Peebles 2004, doi:10.1086/425155, stars about 6 %; Macquart et al. 2020, doi:10.1038/s41586-020-2300-2).
- Dark matter: 37 % in catalogued haloes, 22 % below the floor (to 10⁶ M☉), 40 % never collapsed into those.
- Matter gathers into filaments and knots (51 % and 19 % of matter in 42 % and 9 % of volume) and drains from voids (3 % of matter in 8 % of volume).

## A finding for Alex (question 1 of DM3)
The design's environment check names Cautun et al. 2014 (filaments about half the mass in about 6 % of the volume; voids about 15 % of the mass in 77 % of the volume). Those shares are from NEXUS+, a multiscale classifier. The web here uses the T-web at a zero threshold on its finest octave, which gives far fewer voids (8 % of the volume) and far more filament volume (42 %). The matter in filaments agrees (51 %). The shares depend strongly on the method (Libeskind et al. 2018, doi:10.1093/mnras/stx1976, compare twelve). Raising the T-web threshold to 0.4 gives voids 28 % of the volume and filaments 19 %.
1. My recommendation: keep the T-web, say on the Cosmos page which classifier it is and that NEXUS+ would call more of space void, and leave the threshold where it is until the foam map is judged by eye. A threshold change moves the web's goldens.
2. Alternative: raise the threshold to about 0.4 now (Forero-Romero et al. 2009 found a positive threshold matches the eye better).

## Not yet
- The "where the matter is" bar and the dark matter layer on the foam and cluster maps (galaxy charts thread).
- Per-halo ledgers through time (design 4.2) and energy (4.3) come with GH4 and later.

## Next
GH4 (components in the envelope: streams, shells, ancient patches).
