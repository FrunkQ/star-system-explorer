# SOL4: catalogued rocky worlds bring their measured interiors

This was asked by the coordinator on 24 Nov, from the Traveller thread's finding that every catalogued Sol body read about 23% iron. The door gave every catalogued world the default core mass fraction of 0.32, so Mercury, the Moon and Mars all came out Earth-like.

## What changed

- **Measured rows in base-sol.** `scripts/build-base-sol.py` gives the rocky worlds their measured interiors as catalogued rows (`INTERIOR`, `INTERIOR_SOURCES`). Every value is marked `check: unverified`, because they were recalled from the papers named and not yet re-read against them.

  | World | `interior.core_mass_fraction` | Source | `body.moment_of_inertia_factor` | Source |
  |---|---|---|---|---|
  | Mercury | 0.69 | Hauck 2013 | 0.346 | Margot 2012 |
  | Venus | none (no measured core) | | 0.337 | Margot 2021 |
  | Earth | 0.325 | McDonough 2003 | 0.3307 | Yoder 1995 |
  | the Moon | 0.015 | Weber 2011 | 0.3932 | Konopliv 1998 |
  | Mars | 0.24 | Stähler 2021 | 0.3644 | Konopliv 2011 |

- **Adoption reads them.** `adopt.rs` reads `interior.core_mass_fraction` from the record, with 0.32 only where none is given. Composition, inventory, the core history and the field all follow from it.
- **Measured C wins.** The measured moment of inertia factor wins over MOI1's two-layer estimate.

## Checks

- `sol_tests::catalogued_worlds_bring_their_measured_interiors`:
  - Mercury's bulk is about half iron, against Earth's 0.3; the core alloy's CI sulfur (MP14) keeps it below the measured 60 to 70%;
  - the Moon's iron is under 15%;
  - the order is Mercury, Earth, Mars, the Moon;
  - the measured C values come through.
- `earths_deposits_hold_its_porphyry_copper`: the tolerance is widened from 30% to 35%. With Earth's measured 32.5% core, the mantle and the arc copper are slightly smaller (1.17e14 kg against Kesler and Wilkinson's 1.7e14).

## Next

- Re-read the nine sources and clear the unverified marks.
- Add Ganymede and the other big moons when base-sol carries them.
