# DM4: dark matter inside a galaxy

Track GH (`design/SSE4-galaxy-histories-design.md`, section 5), built 22 Nov 2026 by the kernel thread. This is part 1: the blocks and their checks, and the rotation curve card's data. Part 2 retires the typed speeds.

## Part 1: the profile and the curve (`crates/sse-cosmos/src/structure.rs`, fudge F44)
- `structure::concentration`: the halo's concentration from its own growth. The mean density inside r₋₂ is a fixed multiple of the critical density at the time the main progenitor first held the mass now inside r₋₂ (Ludlow et al. 2014's form). Ludlow's constant suits simulated main branches. With G2's Correa histories the constant that meets Dutton and Macciò 2014 is 800, not about 400. With it, every decade from 10¹⁰ to 10¹⁵ M☉ lands within 0.06 dex (about 0.03 in practice), so only the zero point is calibrated and the mass trend comes out of the physics. An early-forming halo is denser.
- `Structure::new`: the (α, β, γ) profile after the galaxy's response (Di Cintio et al. 2014, from M*/M_halo, strength 0 to 1), normalised so the dark matter inside r200 is the halo's mass less its baryon share. The missing baryons (the share not in stars, cold gas or the black hole) sit in hot gas that follows the dark matter.
- `Structure::speeds(r)`: circular speed by part. The parts are dark matter, hot gas, stellar disc and gas disc (Freeman, Bessel functions by Abramowitz and Stegun's fits, tested against their table), bulge (Hernquist, sized by Shen et al. 2003) and the black hole. Each part is listed with the total and the "visible only" speed, the classic picture of why dark matter was proposed. Also `dark_density(r)` and `v_max()`.
- Settings in `GalaxyParams.structure` (serde default).
- The kernel's `galaxy_chart` gains `rotation_curve`: the structure, 49 log-spaced points out to r200 (at most 100 kpc), the speed at the Sun's radius, the local dark matter density and v_max.

## Checks (tests in `structure.rs`)
- **Milky Way.** The measured visible parts (Bland-Hawthorn and Gerhard 2016) in a median 1.1 × 10¹² M☉ halo give 236 km/s at 8.122 kpc (Eilers et al. 2019: 229), with dark matter 139, disc 155, bulge 84 and gas 60. The curve falls to 206 at 20 kpc and 199 at 25 kpc (Eilers: about 200). Local dark matter is 0.27 GeV/cm³, just under Read's 0.3 to 0.5. The test allows 0.25 to 0.6.
- **Baryonic Tully-Fisher.** Over 10¹⁰·⁵ to 10¹²·⁵ M☉ haloes with G3's galaxies, the slope is 4.0 (Lelli et al. 2016: 3.85 ± 0.09) and the zero point 32 M☉ km⁻⁴ s⁴ (McGaugh 2012: 47 ± 6). This is the observation MOND rests on, reproduced with haloes as the design asks.
- **Concentrations.** Within 0.06 dex of Dutton and Macciò from 10¹⁰ to 10¹⁵.
- **NFW.** With no response, the table matches NFW's analytic enclosed mass to 0.2 %.

## A finding for the galaxy thread
G3's own median 10¹² M☉ galaxy turns at about 182 km/s at 8 kpc, not 229. It has 3.0 × 10¹⁰ M☉ of stars on a 3.1 kpc disc; the Milky Way has 5.7 × 10¹⁰ on 2.6. The dark matter part (121 km/s) is fine. The test `g3_s_own_milky_way_like_galaxy_turns_more_slowly` records this. It is not a target. It matters for part 2: if the typed 220 km/s is retired in favour of the default universe's own galaxy, the Sun's orbit slows unless the home galaxy carries the Milky Way's measured parts (GH7's pins).

## Goldens
The kernel contract golden is re-blessed on purpose. Its two `galaxy_chart` responses gain the `rotation_curve` field, and nothing else in them changes. The G3 golden is untouched, because the galaxy itself is not changed by part 1.
