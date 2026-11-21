# O3 patch side (light track): how many nebulae a patch expects

The galaxy patch (`sse_cosmos::patch`) now counts the glowing nebulae it expects without placing them, so a
chart of an unvisited region can say what it holds before any star is resolved (nebulae and light design 4).
The kernel thread handed patches to the light track for this; `nucleus.rs` and `galaxy.rs` are untouched.

## What was built
- `PatchModel.glows` (pack data, all defaulted so existing packs load): main-sequence life rule (the star
  settings' 10 Gyr × M^−2.5), the Kalirai et al. 2008 initial-final mass relation, the ionising-star floor
  (15 M☉) and the mean ambient gas density (10⁶ H per m³).
- `Conditions` gains: white dwarf birth rate per pc³ per Gyr, mean shed envelope, ionising stars alive per pc³,
  and expected planetary nebulae, core-collapse and thermonuclear remnants, and ionising stars in the volume.
  Each cohort forms its stars steadily over its span, so stars die now if their lifetime lies between the
  cohort's youngest and oldest ages (IMF share in that mass range, 64 log bins).
- Visible lives come from `sse-nebula`: new `planetary_fades_after` (shell thinned to `fade_contrast` times its
  surroundings, the same rule `planetary_nebula` uses, now shared) and `expected_remnants`.
- Patch facts `patch.wd_birth_gyr_pc3`, `patch.ionising_stars_pc3`, `patch.expected_planetary_nebulae`,
  `patch.expected_cc_remnants`, `patch.expected_ia_remnants`.
- Air gases (for the renderer's V3 mixes): ammonia, sulphur dioxide, carbon monoxide and neon in the base-light
  gas table.

## Evidence (`cargo test -p sse-cosmos glows_near_the_sun`, `cargo test -p sse-light v3_mix_gases_resolve`)
- White dwarf birth rate near the Sun 7.1×10⁻¹³ per pc³ per year, against about 1×10⁻¹² (Liebert, Bergeron and
  Holberg 2005, ApJS 156, 47, doi:10.1086/425738).
- Planetary nebulae 23 per kpc³ near the Sun, against about 25 (Frew 2008, PhD thesis, Macquarie University,
  from memory, verify); implied visible life about 32 000 years.
- Mean shed envelope 1.6 M☉; a patch whose young cohorts are removed has no ionising stars and no new remnants
  but still has planetary nebulae.
- Neon scatters under a tenth of air, sulphur dioxide over four times air.

## Goldens
`tests/golden/cosmos.json` and `tests/fixtures/cosmos/solar-patch.json` change on purpose: the patch's
conditions and facts gain the glow counts. Nothing else moves.

## Limits (docs/fudges.md, kernel light track)
One ambient density for every patch; ionising stars stand in for H II regions (which group many stars);
King factor 1 for the four new gases. Next: the region resolver places the expected nebulae (O3 later slice).
