# GZ12: home is one of the cosmic web's own galaxies (26 Nov 2026)

Design: `design/SSE4-grand-zoom-design.md` 6.3 point 1, 6.5, 6.6; accepted 25 Nov (14A). Status: `status/grand-zoom.md`.

## What it does
- **The choice.** `sse-cosmos/src/home.rs`: the home is the catalogued halo (G2) that meets every wish of the cosmology pack's `home` settings and is nearest the middle of the wished mass range (in decades), over every cell within `search_shells` (2, so 125 cells) of cell 0.0.0. Ties go to the nearer shell, then the address. With no fit it takes the halo nearest the wished mass, marked `fits_wish: false`, and never refuses. Only haloes above `mass_min` are read (`Haloes::cell_above_full`: exactly `cell`'s haloes above a floor, with mergers and neighbours). It costs about 0.3 to 0.6 s native for 125 cells, worked out once per universe and kept (`cosmos.rs` `HOMES`, a cache of a pure answer).
- **The wish** (`packs/base-cosmology` `cosmology.home`): the Find a home screen's "Like the Milky Way" (0.5 to 2 × 10¹² M☉; no major merger for 8 Gyr; a bound bigger neighbour within 1 Mpc), plus `neighbour_mass_ratio_max` 3: the partner is Andromeda-like, not a group 10 times our mass that we would be a satellite of. The first draft picked a 1.2 × 10¹³ group's satellite for g3. `disc_kpc_min/max` exist as an optional ranking preference and are off by default.
- **Who reads it.** `galaxy_chart` with no galaxy named, or `halo: "home"` (the chart then carries `home`); `with_world` and `with_stars`, so `star_cell`, `sky_from_here`, `sky_glow`, `promote_star`, `from_star` births and `plan_voyage`; and `reconstruct_origin`. The sky looks out from the home halo's own place (`galaxies_seen`: home = host position, and the host is skipped by address). `SkyParams.home_mpc` is gone. The named reference galaxies (dwarf, milky-way-like, group) stay, by name only.
- **Make this home.** `set_home {halo}` writes `universe.home_halo` on the universe's root as one record event. `set_home {}` clears it, back to the web's choice. A halo that is not one of this universe's catalogued haloes is refused, and nothing is written. `home_galaxy` reads the home with its wish and its reasons. Session methods `homeGalaxy` and `setHome` are in `ui-contract/session.ts`, for the galaxy charts thread's button.
- **UI.** The galaxy chart's default is "home" ("Our galaxy" in `NAMED_HALOES`), so going up from a system lands in the home galaxy. That closes test-drive rough edge 3 for every system, since all systems sit in home until GZ13 gives each one a slot. Galaxies nearby is centred on the real home through the sky.

## Bugs it found and fixed
1. **NaN progenitor masses** (`assembly.rs` `mass_today_for`). For a progenitor that fell in at z above about 12, the fixed-point inversion of the median growth ran away to a degenerate Correa shape and returned NaN. The host's infall budget then went NaN, its gas starved, and its whole fed history went NaN: g3's first home had no stars at all. Now, when the iteration does not land on a root, the root is found by a scan in 0.05 dex steps and bisection. With no root (bigger than any median halo was then), it takes the nearest. Answers that were already right are unchanged. Gate: `gz12_tests::an_early_forming_home_grows_a_whole_galaxy`, seen red before the fix.
2. **Merger-brought stars left dark in the sky** (`galaxies.rs` `look`). The light came from the star formation history only, so ex-situ stars (most of a group central's) were dark: M/L_V 13. They are now lit as if they formed like the galaxy's own stars (the history scaled to the galaxy's stars), fudge F37.

## The sky's Milky Way checks (fudge F37b)
Four calibration checks (stars brighter than V 3 to 5, our satellites, the brightest neighbour's light, the glow) were tuned on the one median stand-in galaxy. On real default homes they scatter:

| Universe | V < 3 (real 171) | Fails |
|---|---|---|
| g3 | 66 | counts, satellites |
| sse4 | 106 | counts, satellites, chart against sky mass |
| gamma | 304 | counts, glow dust lane |
| beta | 339 | counts, glow dust lane |
| alpha | in range | glow centre against anticentre (by 0.003 mag) |

The cause: the Sun's place is 8.2 kpc × R_d / 2.6, set by scale length alone, not by local stellar density. No home passes all four, so none was picked (that would be cherry-picking). The coordinator decided (26 Nov, under Alex's ruling that the coordinator decides routine test changes) that the four checks run on the median reference galaxy through `cosmos::use_reference_home()`. That switch is `cfg(test)` and cannot be reached from the product. Every limit is unchanged, and the glow relaxation Alex approved turned out not to be needed. Beside them, `home_tests::every_default_home_is_a_whole_lit_galaxy_within_its_wish` checks six universes' real homes: the wish's bounds, a finite history with star formation, a lit sky with M/L_V under 10.

**Follow-up package GZ12b** (for an owner beside the chemistry retune): the Sun's place from the local stellar density (with GZ13's star slots), and the glow's structure per galaxy. After that, the four checks move back to the real home.

## Tests and goldens
- New: `sse-cosmos` `gz12_tests` (4) and `sse-kernel` `home_tests` (4); golden area `grand-zoom`.
- Edited to follow the design: `tests/ui/galaxy.e2e.mjs` and `tests/ui/place.e2e.mjs` (the galaxy chart's first place is "home", "Our galaxy", and a catalogued address, not the median), with Alex's yes. `names_tests` (a promoted star is taken from the home cell, not a fixed address in the old galaxy) and the sky test's cell parse (the star cell is the last cell in the address), with Alex's yes.
- Core golden: `tests/golden/contract.jsonl` moves on purpose. The default galaxy and its star addresses change, and so does everything grown from them.

## Left for the galaxy charts thread
- `ui-contract` `galaxy.ts` `HOME_CELL` is still `[0, 0, 0]` (it says "if GZ12 moves home, this follows it"). The home is now in any cell within two of 0.0.0 (g3's is 0.2.-2), so "Milky Way neighbourhood" should follow `homeGalaxy().cell`. That is the Galaxies nearby screen's change, with "Make this home".
