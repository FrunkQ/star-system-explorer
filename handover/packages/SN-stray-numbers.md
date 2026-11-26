# SN: stray physics numbers moved to where the rule says they live

Thread "Move stray physics numbers to the sheet" (opened 26 Nov 2026 by the coordinator after the audit
"Check today's physics is in blocks"). Alex's rule: all physics in editable, cited blocks; all material and colour
data on the master data sheet, read through Look up; shortcuts in `docs/fudges.md`. One kind of work: moving typed-in
numbers to where they belong. Values are kept as they were unless a part below says otherwise.

## SN1: lifetimes in sunlight (water, CO2, CO)

- Sheet: `substances.<id>.photochemistry.lifetime_1au` on h2o (8.3e4 s), co2 (5.0e5 s) and co (1.3e6 s), quiet Sun
  at 1 au, growing as the distance squared; source `huebner1992`, every value **unverified** (recalled survey values,
  as both readers had them; the paper's tables not read, Consensus closed until 1 Dec 2026). Built by
  `scripts/photo_rows.py`; the compiled copy is `sse_core::sheet::photo_lifetime_1au`.
- Readers: the ice ground block `sse.surface.open_ice_loss` loses its `photo_rate_1au` parameter (1.2e-5 per second
  typed in the block and in `sse-atoms`) and takes the lifetime as an input, kind `material.photo_lifetime_1au`
  (Look up); `sse-surface` passes the sheet's water row. `sse-small` fills each comet volatile's lifetime from the
  sheet at load, and `packs/base-small-bodies` no longer types them.
- **Value change (rounding):** the ice ground's rate was 1.2e-5 per second; it is now 1 / 8.3e4 s = 1.205e-5 per
  second (+0.4 %). Both were the same two-figure fact; the sheet keeps one, the lifetime as the comet pack had it.
- **Golden:** one core contract line moves (`system_resolved` for Kestrel sys 40): four ice grounds' `lost_km` grow
  by 0.4 % and the `why` text's metres with them. Nothing else changes.

## SN2: nebular and comet ion lines on the sheet

- Sheet: four ion rows with `optical.emission`, beside sodium's and potassium's lines (`scripts/emission_rows.py`):
  `o+2` ([O III] 495.9 and 500.7 nm), `s+` ([S II] 671.6 and 673.1 nm) with every nebular number PR 588 typed into
  `packs/base-light` (collision strengths, weights, Einstein A, decay shares, ion energies, the S II balance fit),
  moved unchanged; `co+` (about 426 nm) and `h2o+` (about 615 nm), the comet ion tails' colours (Brandt and Chapman
  2004, unverified). Sources split into sheet source rows (storey2014, sii-aa2025, verner1996, badnell2015,
  nist-asd-ion, storey-nlevel, brandt-chapman2004), all unverified.
- Light pack: those four lines keep id, name, species and matter and gain `sheet` (the row); `at_m` and `nebular`
  come from the sheet (sse-light `line_from_sheet`; the nanometres read to metres exactly as written, so no
  wavelength moves by a bit).
- Renderer: `comet.ts` reads the tail's line from `sheetLine` (ui-contract, generated `lines` from the sheet) and has
  no typed wavelengths; `sheetLine` throws on a missing line.
- Not moved: the light pack's `photon_takers` edges (hydrogen's and helium's ionisation energies) and the other
  lines' wavelengths (no atomic data with them). Left for a later pass.
- **Golden:** no number changes. The nebulae golden and three core contract lines (two `light_read`, one
  `nebulae_read`) change in their reference text only (each now the sheet source's text and note, with its own DOI
  or link, where the old text had mangled the S II DOI into a URL), and `light_read` lines gain `sheet`.

## SN3: life on a world of the chart (L3, PR 626)

- Fudge rows L3-sketches (every Earth-scaled supply in `life_from_world`), L3-dose, L3-grades and L3-vents in
  `docs/fudges.md`.
- Rock conductivity from the sheet: `crust-rock.thermal_conductivity` 2.5 W/m/K (Turcotte and Schubert 2014,
  unverified; the figure the pack's note already gave). The pack's `warming_at_depth_k_per_w_m2` 2000 becomes
  `warming_depth_m` 5000, and the life suite divides by the sheet's conductivity: the same 2000 K per W/m².
- Graded: the sea's vent iron as 1/(1 + O2/1e-5) (the air box's own blend, F4), and reducing air as
  x^4/(x^4 + 0.1^4) in the H2 share, with methane alone weighing lr_ch4^(1-s) x lr_nothing^s. Half points are the
  old thresholds; the steepness 4 is authored. Values move only near the old edges (Earth's air keeps about 5e-5 of
  the Archean vent iron, which the tests do not feel).
- **Not graded: vents.** Grading them cut Earth's vent hydrogen by about a tenth, because the interior gives Earth
  0.039 W/m² against the measured 0.09. That moved Earth's oxygen rise 80 Myr later (4205 against 4125 Myr) and its
  land life to 4555 Myr, after the timeline's last point, so `catalogued_earth_grows_land_life_and_its_own_oxygen`
  failed. The cause is the low heat flow, so the switch stays (fudge L3-vents) and the test was not changed.
- Goldens: none move.
