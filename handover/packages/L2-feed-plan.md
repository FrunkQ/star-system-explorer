# L2-feed (plan, not built): the air box reads what life makes, not what light allows

Track LE. Written 27 Nov 2026 by "Build life's energy baseline" after L3-heat. The coordinator chose option A on 27 Nov: build F1, F2 and F3 as one set, held until Earth oxygenates honestly.

## The cause of the fast oxygen

`history::StepFluxes::by_metabolism`, which the air box reads for burial and methane, is filled from `HabitatBudget::by_metabolism`, which holds each metabolism's **energy cap**, not its production. The sunlit ocean is phosphorus-limited, so its oxygenic cap (1.16e17 g C a year on the catalogued Earth) is four times what it makes (2.9e16). Burial is 0.1% of the cap: 9.7e12 mol a year against 4.3e12 of reducing gases, so the air reaches 14% oxygen within 50 Myr. This is the register's F1 ("undocumented error, not a fudge"), and F27 already says E3's brakes were set on top of it. The biosphere already shares production correctly (`biosphere.rs`, best light metabolism only, in proportion to the caps).

## The F1 part (written, held on a branch)

- `budget::production_by_metabolism(h, bp)`: one rule, used by both the biosphere and the history's step fluxes. The soil's NPP reads production too.
- There are no new numbers.

## What it does (seed "home", correct tilt)

- After oxygen photosynthesis at 2105 Myr, the air stays near 2e-5 oxygen (1e-4 of today's) with methane near 800 ppm for about 1.8 Gyr. Burial is 2.4e12 mol a year, against reducing gases falling from 4.3e12 to 2.8e12 along the stand-in e-fold (F2), and hydrogen escape makes up the difference. It reaches 0.23% at 3950 Myr and never 1% by today. Complex cells arrive at 3965 Myr and multicellular life at 4240. There are no large bodies and no land.
- The fixture Earth never oxygenates either. Red: `feedback_tests` (oxygenates, competing sinks, golden), `soil_tests` (two), and the kernel's `catalogued_earth_grows_land_life_and_its_own_oxygen` and `life_view_carries_the_energy_ledger`. The core contract does not move.

## Why the physics cannot give today's air yet

With burial at the cited 0.1% of production, marine burial sits below the cited reducing gases (Catling and Claire 2005: 3e12) until they fall. Three missing pieces carry the real rise, all already in the register:

1. **F3, pyrite burial.** Sulfate reduction burying pyrite is about 7.8 Tmol O2 a year, about 40% of modern sources (Catling and Claire 2005). It is absent.
2. **F2, the fall of the reducing gases.** In the box it is an e-fold stand-in. In the sources it is gases growing more oxidised as the crust is oxidised by hydrogen escape (Catling et al. 2001; Claire et al. 2006, the "hydrogen hourglass"). The box books escape beyond the reducing gases as `escape_to_crust` and throws it away, so it never lowers later gases.
3. **Reducing gases by heat.** These should follow the world's heat by age (L3-heat's samples) and the heat-lost door, rather than today's heat times the e-fold.

## Options

- **A (chosen by the coordinator, 27 Nov).** Ship the planned PR now (L3-air, L3-heat, loader), saying plainly that Earth's dates still stand on F1. Then build L2-feed as F1 + F2 (escape oxidises the crust, gases follow heat) + F3 (pyrite), from the sources already read (Catling and Claire 2005, Catling et al. 2001). Numbers not yet read wait until the paper search returns on 1 Dec, and are registered as fudges until then. Hold the F1 fix on a branch until the set passes honestly.
- **B.** Ship F1 alone: not possible, it turns seven tests red, and they are not loosened.
