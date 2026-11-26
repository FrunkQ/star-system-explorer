# K18 describe_star, and three fixes asked on 25 Nov

Status: done (25 Nov 2026), kernel track.

## describe_star (editing thread, ED6 part 2, editing design section 9)
- `{"op": "describe_star", "mass_msun", "age_yr", "feh"?}` returns `star_described` with
  `sse_formation::describe::Description`. It gives:
  - the phase now, with `teff_k`, `luminosity_w`, `radius_m` and `mass_kg` (solar units alongside);
  - each phase's start and end ages in years;
  - the remnant and its mass;
  - the mass `walls`: deuterium 0.0124, hydrogen 0.08, white dwarf 8, black hole 25 and the fits' 100 M☉;
  - notes, such as a Z off the fits or the 0.08 to 0.1 extrapolation.
- It reads the same life line formation draws: `LifeLine::new` and `state_at` on `Fits::new(Z)`.
  Z comes from [Fe/H] through the new `inherit::z_of_feh`, shared with formation, so the numbers are
  unchanged. Test `formation_draws_the_same_star`.
- Read-only, with no events. Contract cases are in `tests/contract/areas/kernel.json`.

## Fixes
- **Grown systems were 13,783 Gyr old (galaxy charts thread):** `cosmos::with_stars` gives the
  galaxy's now in Myr, and from_star handed it on as Gyr. The core history then stepped millions of
  times, and wasm ran out of memory. Test `a_grown_system_is_as_old_as_its_star`.
- **The `systems` list missed grown stars:** `adopt::systems_of` now takes the outermost `sys`
  segment at any depth under the root, so `u:x/halo:…/cell:…/sys:…` is listed.
- **class_sample addresses carry the branch and class** (`sys:class-sample-<n>-<8 hex>`), so a surface
  cache keyed by address never shows one call's world for another (renderer thread).

## Open
- Small-body samples (Eros, Bennu, 67P, Psyche, Arrokoth) come back with `radius_m` null. The atlas
  examples give mass, makeup and porosity but no radius, and the resolve does not size them. K3's
  radius from mass and makeup should cover small bodies, with porosity. The renderer skips these for
  now. It reported that a level-0 tile read for 67P ran for minutes in the browser; natively the same
  read returns at once.

## Goldens
`tests/golden/areas/kernel.jsonl`:
- the three describe_star lines;
- the class_sample line, whose addresses changed.
The core golden is unchanged.
