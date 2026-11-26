# SSE4: the check set for the galaxy's ledger (CC2)

Status: **draft for the builder, 26 Nov 2026.** Written by the thread "Design the conserved budget chain" under Alex's "yes to all" of 14:52 UTC (note `SSE4-conserved-chain-design.md` section 13, item 4). Design only: this file sets what the rings model must match; it changes no code. Package CC3 retunes against it.

**Every figure below is from memory and unverified.** Consensus is out of its monthly allowance until 1 Dec 2026, so no source here was read today. Per the build plan's rule, a builder checks a row against its source before relying on it, and the citations thread scores it on the master list. Rows already on the master list are marked (ML).

## 1. Why a check set

A ledger that closes can still be wrong. The rings' books balance exactly and the gas still came out too rich in iron, because two tuned settings (yields, radial winds) were fitted to one measurement, the slope of the gradient between about 5 and 17 kpc. Once every star and disc claims its recipe and amount from its ring (CC6), an error here reaches every system below it. So the ledger is held to a set of measured things at once, not to one.

## 2. Three findings before the table

1. **The level was never checked, only the slope.** `g3_tests.rs` tests the slope of gas [Fe/H] between 4 and 14 kpc, and the Sun's birth gas to within 0.2 dex. Nothing tests the gas today at the Sun or anywhere else. The Local Arm thread's numbers (home galaxy, Sun at 9.88 kpc, disc scale length 3.13 kpc, with its bar sweep): gas [Fe/H] now +0.46 at 4 kpc, +0.35 at 5, +0.27 at 6, +0.21 at 7, +0.16 at 8, +0.09 at the Sun, -0.02 at 13. That slope passes. The whole line sits about 0.1 dex high.
2. **Anchor the level on young stars, not on gas clouds.** Oxygen measured in H II regions from their electron temperatures runs about 0.15 to 0.25 dex below what young stars in the same places give (the long-known abundance discrepancy). So "0.15 to 0.2 dex above the measured H II regions" is partly that offset and not all model error. Use B stars and Cepheids for the level, and H II regions for the slope only.
3. **Radii are in disc scale lengths.** The home galaxy is not the Milky Way's size. Every radius below is given in Milky Way kpc and is to be scaled by the galaxy's own scale length over 2.6 kpc, as `mw_kpc` in `g3_tests.rs` already does. The Sun's place is 8.2 kpc, 3.15 scale lengths.

## 3. Tier A: must hold before CC6

| # | Measured thing | Value and tolerance | Source (from memory) | Today |
|---|---|---|---|---|
| A1 | Gas and young-star [Fe/H] at the Sun's radius, now | 0.00, within 0.05 | Nieva and Przybilla 2012 (nearby B stars are solar); Cepheids at the solar circle | +0.09: fails |
| A2 | Gas [Fe/H] at the Sun's birth, 4.57 Gyr ago, at the Sun's birth radius | 0.00, within 0.10 | By definition of solar | Tested to 0.20; tighten |
| A3 | Slope of young-star [Fe/H] with radius, 5 to 15 kpc | -0.06 dex per kpc, within 0.01 | Genovali et al. 2014 (ML) | Passes (band -0.08 to -0.03; tighten) |
| A4 | Level along that line: [Fe/H] at 5 kpc | +0.19, within 0.07 (A1 plus A3) | As A1 and A3 | About +0.3 at the scaled radius: fails |
| A5 | Inner flattening: young-star [Fe/H] inside about 4 kpc | No higher than +0.3 anywhere; no steepening inward of 5 kpc | Genovali et al. 2013 (inner-disc Cepheids); Martin et al. 2015 | +0.46 at 4 kpc after the bar sweep, +0.47 to +0.84 before: fails |
| A6 | Young stars at the centre (inner 200 pc) | [Fe/H] between -0.1 and +0.2 | Davies et al. 2009; Najarro et al. 2009 (red supergiants and the Quintuplet) | Not tested |
| A7 | Slope of gas oxygen with radius, 5 to 17 kpc | -0.04 dex per kpc, within 0.01. Slope only, not level | Arellano-Cordova et al. 2020; Esteban and Garcia-Rojas 2018; Mendez-Delgado et al. 2022 | Not tested (alpha group) |
| A8 | Gas surface density at the Sun | 13.7 solar masses per square parsec, within 20% | McKee, Parravano and Hollenbach 2015 (ML) | Local Arm thread: "looks right"; not a test |
| A9 | Star surface density at the Sun, remnants included | 33 solar masses per square parsec, within 20% | McKee, Parravano and Hollenbach 2015 (ML) | Not tested |
| A10 | Visual extinction in the plane from the Sun to the centre | 30 magnitudes, within 5 | Nishiyama et al. 2008; Fritz et al. 2011 (near-infrared extinction to the centre, converted) | 46: fails. About 21 with metals held solar |

A1, A4, A5 and A10 fail together and point the same way: too much iron in the gas at every radius, and more so inward.

## 4. Tier B: watched, reported, not yet a gate

| # | Measured thing | Value | Source (from memory) |
|---|---|---|---|
| B1 | Total stellar mass | 5 to 6 x 10^10 solar masses | Bland-Hawthorn and Gerhard 2016 (ML); Licquia and Newman 2015 (ML) |
| B2 | Star formation rate now | 1.65, within 0.4, solar masses a year | Licquia and Newman 2015 (ML) |
| B3 | Disc scale length | 2.6, within 0.5, kpc | Bland-Hawthorn and Gerhard 2016 (ML) |
| B4 | Stars near the Sun below [Fe/H] of -1 | 1 to 2%; test at under 5% | Already in `solar_neighbourhood_history` |
| B5 | Age against metals near the Sun | Flat for stars younger than about 8 Gyr, mean -0.1 to 0.0, scatter about 0.2 dex; falling for older stars | Casagrande et al. 2011; Bergemann et al. 2014 |
| B6 | Alpha against iron | Plateau of +0.3 to +0.4 below [Fe/H] of -1, reaching 0 at solar | Already tested (0.25 to 0.5) |
| B7 | Supernova rates | Core collapse 1.6, within 0.5, a century; type Ia 0.54, within 0.12 | Already tested (Rozwadowska et al. 2021; Li et al. 2011) |
| B8 | Shape of the gas with radius | Molecular gas peaks in a ring near 4 to 5 kpc; atomic gas near 5 solar masses per square parsec and flat to about 13 kpc; little gas inside 3 kpc apart from the central zone | Heyer and Dame 2015; Kalberla and Kerp 2009 |
| B9 | Scatter of r-process elements | About 1 dex below [Fe/H] of -2.5, shrinking to about 0.1 dex at solar | Already tested in spirit (`r_process_is_lumpy`) |
| B10 | Dust for a given column of gas and metals | Dust rises in step with metals above about a fifth of solar; one magnitude of V per 2.2 x 10^21 hydrogen atoms per square centimetre at solar | Remy-Ruyer et al. 2014; Guver and Ozel 2009 |

Tier B rows are printed by the same test run. A row moves to tier A when its source has been read and its tolerance agreed.

## 5. Advice to the builder (CC3), not rulings

- **Do not tune by radius.** A setting per ring would pass every row and explain nothing. Find the physical cause of the inward rise and fix that.
- **Likely cause of the inward rise.** Ring wind loading is the galaxy's times exp(k (R - R_ref)), with k = 0.6 per scale length (`eta_radial`). Inward of the reference radius it falls towards zero, so the inner rings keep nearly everything they make and run away as closed boxes. Real inner discs lose metals too. A loading that levels off inward, in place of one that keeps falling, is the first thing to try. It should be a cited form, not a floor picked to pass A5.
- **Likely cause of the level.** The iron yields (`yield_fe_cc` 0.7, `ia_fe` 540, `ia_per_mass` 2e-3) set the level everywhere. A1 and A2 together pin them: A2 alone, at 0.2 dex, let the level drift.
- **The bar.** The Local Arm thread's bar sweep already cut the inner excess from +0.84 to +0.46. Mixing along a bar flattens a gradient inside the bar's reach; whether the sweep mixes metals as well as moving gas is worth checking.
- **A10 is the downstream check.** If A1 to A6 pass and A10 still fails, the fault is in the dust rule or the geometry, not the chemistry. Dust rising in strict step with metals above solar is itself uncertain (B10).
- **Golden.** This moves the envelope, sky, glow and galaxy chart lines in the core golden. One move, with the Local Arm PR, in the core lane.
- Each new assertion is first run against today's code and seen to fail (A1, A4, A5, A10 should; A3 should pass).

## 6. What this does not cover

Other galaxies (the mass against metals relation is already tested in `mass_metallicity_relation`), the thick disc and halo, the bulge's own stars (a broad spread from about -1 to +0.5, Ness et al. 2013, which a single-zone inner ring will not reproduce), and anything below the ring. Check sets for the other ledgers (web cell, halo, a system's disc) follow the same form when their joins are tied.
