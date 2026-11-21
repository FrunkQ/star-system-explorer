# SSE4: processes, not classes

**Status: accepted 21 Nov 2026 (Alex: "do the new rules - LEAN INTO THE PHYSICS!").** This is a standing rule for every physics track, starting with surfaces (S0b). From the exogeology review thread. Audit made against `main` on 21 Nov: `packs/base-surface/pack.json` and `crates/sse-surface/src/`.

## 1. The rule

A world does not know what class it is. It has a mass, a composition, a heat flow, a surface temperature, a pressure, a spin and a history, and its features follow from those. So:

1. **A process works out its own strength from physics.** Each process (ice-shell convection, tidal cracking, plate motion, heat-pipe volcanism, N2 glacier flow, dune saltation, tholin fallout and so on) returns a strength from 0 to 1. The strength comes from the physical ratio that governs it, for example a Rayleigh number over its critical value, a stress over a yield strength, a vapour pressure over the ambient pressure, or a supply rate over a removal rate.
2. **Every process runs on every world, weighted by its strength.** The results add into the one height map, the one cover map and the one record of ages. A process whose strength comes out near zero is skipped. That is the "best calc is no calc" rule from S16, so it costs nothing.
3. **The class is a label read off the result, never an input.** The class atlas (C track) names the world from its process strengths and outputs ("ice-shell world", "mobile lid"), two-way and editable. Changing a class's name or boundaries in the atlas never changes the physics.
4. **Hard edges only where nature has one.** Those are phase boundaries (melting, boiling, triple points, condensation), computed from the actual composition and pressure, never typed in as one temperature. A force balance that genuinely flips, such as gravity against material strength for roundness, also counts. Even a real edge gets the width nature gives it, for example the melt fraction between solidus and liquidus.
5. **Every curve says why.** A strength block cites its relation and shows its inputs in "Why?". For example: "ridges 0.7: tidal stress 85 kPa against an ice strength of about 100 kPa (Kattenhorn and Hurford 2009)". Where no published relation exists, the curve is a fudge and goes in the fudge register with its reason.

**Shape of a strength block.** Use `strength = smoothstep` or a logistic of `log(ratio)`, centred on the ratio's critical value. The width comes from the spread in the measurements cited. Where there is no spread to go on, the default width is a factor of 2 either side. Keep the curves deterministic and in `sse_core::math`, as for every other block.

**Saves and hashes.** The `regime` field stays for one version as a derived label: the strongest process family, so old saves and the UI keep working. Golden hashes change once, on purpose, in S0b, with this document as the reason.

**Why this is better, not just tidier:**
- Edge cases stop being bugs. A world near a cut-off gets a mix, as real ones do: Pluto has volatile glaciers, old cratered highlands, possible cryovolcanoes and a tectonic history all at once.
- The exogeology probe's misclassifications go away without special cases. Pluto, Charon, Ceres and Callisto stop getting Europa ridges, and Enceladus and Miranda stop being lumps.
- "Why is this here?" is always answerable, because each feature carries the process and the numbers that made it.
- Orac can tune worlds by physics ("raise the tidal heating") instead of by forcing a class.

## 2. Audit of today's hard thresholds

This covers the surface pack and code as built. **Edge** means real physics, to be kept but computed rather than typed. **Smooth** means it is to be replaced by a strength block. **Fudge** means it stays for now, in the register with a reason, until the stage that replaces it.

### 2.1 Regime rules (`surface.rules`: first match wins, which is the root problem)

| Today | Verdict | Replacement physics |
|---|---|---|
| `cloud-deck`: deep_envelope ≥ 1 | Edge | No solid surface is reachable when the pressure at the rock or ice boundary is beyond the envelope's critical point. Compute it from envelope mass and composition; giants and mini-Neptunes then share one rule. |
| `magma-ocean`: molten ≥ 1 | Edge with width | Melt fraction between solidus and liquidus of the actual mantle composition at the surface temperature and the heat of formation. A partly molten surface comes out naturally. |
| `lava`: t_surface ≥ 1400 K | Edge with width | The same melt-fraction block, using the crust's own solidus (basaltic and ultramafic crusts differ by about 100 to 200 K). The day-night split already in the sketch stays. |
| `small-body`: radius ≤ 300 km | Smooth | Roundness from self-gravity against material strength: relief allowed ∝ strength / (ρ g R). The "potato radius" is about 200 km for ice and 300 km for rock (Lineweaver and Norman 2010, arXiv:1004.1091). Relief shrinks smoothly with size, so Enceladus and Miranda come out round but rough, and Mimas comes out round. |
| `hydrocarbon`: T 70 to 130 K, P ≥ 0.5 bar, water ≥ 0.05 | Smooth plus edge | The liquid is stable where T lies between the melting and boiling curves of the actual CH4, C2H6 and N2 mix at that pressure (CH4 triple point 90.7 K, 0.117 bar). Strength = hydrocarbon supply (tholin photolysis, outgassing) × liquid stability. The water condition is dropped, because the crust's material is its own input. |
| `ocean-world`: T ≥ 260 K and water ≥ 0.05, or oceans ≥ 4 | Smooth | Sea level comes from water volume against the hypsometry (the shape of the height map). Land fraction is then continuous, and "no land" means water depth exceeds the relief, which itself depends on gravity and strength. High-pressure ice at the sea floor is a phase edge. Liquid follows water's phase diagram, which `cover.rs` already uses for its triple point of 611.657 Pa. |
| `ice-shell`: water ≥ 0.05 | Smooth | Three separate strengths. **Ice at the surface** is a phase edge from surface temperature. **Shell activity** is the convection Rayleigh number over its critical value, plus tidal stress over ice yield strength. **Subsurface ocean** comes from the heat balance: conductive shell thickness compared with the water inventory. Water fraction feeds the layering only. This is the main S0b fix: Callisto (inactive), Europa (active) and Pluto (volatile-dominated) separate by physics. |
| `heat-pipe`: q ≥ 1 W/m² | Smooth | The fraction of heat lost by advection (eruption) against conduction through the lithosphere (Moore and Webb 2013, Nature 501, 501, doi:10.1038/nature12473). It rises continuously with heat flow and melt supply, so Io comes out heat-pipe and a young Earth partly so. |
| `carbon-rich`: C/O ≥ 0.8 | Edge with width | Mineralogy from equilibrium condensation. Silicates give way to carbides and graphite across a transition near C/O 0.8 to 1.0 (Bond, O'Brien and Lauretta 2010, ApJ 715, 1050, doi:10.1088/0004-637X/715/2/1050). Mineral fractions come out continuous. |
| `super-earth-uncertain`: mass ≥ 4 M⊕ | Smooth plus honesty mark | Compute lid mobility as for any rocky world. Add a "disputed" mark whose weight grows with mass, because research on plates for super-Earths is split. The mark is information, not a separate class. |
| `mobile-lid`, `episodic-lid`, `stagnant-lid`: mass ≥ 0.3 M⊕, q_ratio ≥ 1, oceans ≥ 0.05 | Smooth | One lid-mobility strength: convective stress against lithosphere yield strength, with water lowering the yield (damage and grain size, Foley, Bercovici and Landuyt 2012, EPSL 331, 281, doi:10.1016/j.epsl.2012.03.028). Low mobility gives a stagnant lid, middling gives episodic, high gives mobile (Moresi and Solomatov 1998; O'Neill et al. 2016). Over time it is a continuum, so a world can drift from one to another as it cools. |

### 2.2 Other numbers in the surface pack and code

| Today | Verdict | Replacement physics |
|---|---|---|
| `q_crit_w_m2` 0.019 (active geology) and volcanism `q_min_w_m2` 0.02 | Smooth | Melt production: mantle potential temperature against the solidus, which gives decompression melting. Resurfacing rate follows melt volume. |
| Landform activity `(q/q_ref)^3`, capped at 1 (regimes.rs, "heat" and "heat-now" drivers) | Fudge, then smooth | The cube has no cited basis. Replace it with the process strength of the landform's own driver: tidal stress for ridges, convection for chaos and coronae, extension rate for bands and rifts. |
| Crater renewal from heat flow alone (impacts.rs `renewal_survival`) | Smooth | Renewal rate = the sum of active resurfacing processes: volcanism (melt volume), ice convection (cell overturn), volatile cycling (M1), sedimentation and erosion (wear). This also gives Sputnik its age of under about 10 Myr, from N2 convection. |
| Crater simple-to-complex at 15 km, peak ring at 150 km, multi-ring at 600 km | Already scaled | They are multiplied by a gravity factor `gs` in impacts.rs and atlas.rs, which is right (transition ∝ 1/g). Add a target-strength factor for ice against rock, since icy targets make the transition at smaller sizes. |
| `wear.dune_pressure_bar` 0.005 | Smooth | Saltation: wind friction speed against the threshold friction speed, which depends on grain size, grain density, gravity and air density (Iversen and White 1982, Sedimentology 29, 111, doi:10.1111/j.1365-3091.1982.tb01713.x). The present cut-off would forbid Pluto's dunes, which exist at about 10 µbar (Telfer et al. 2018, Science 360, 992, doi:10.1126/science.aao2975). |
| `land_ice_k` 266, `wear.ice_k` 263, `cold_trap_k` 165, `ice_dry_k` | Smooth | Land ice from mass balance: snow supply against melt and sublimation. Cold traps from the water vapour pressure against the age (loss rate × time). Both come from vapour-pressure blocks M1 already needs. |
| `sea_ice_k` 271 | Edge | The freezing point of the sea, which depends on salinity (271.3 K for Earth sea water). Compute it from the ocean's salt, once oceans carry composition. |
| `shadow_ice_max_bar` 0.001 | Smooth | Polar shadow ice survives where sublimation loss is below supply and the air cannot carry heat into the shadow. A vapour-pressure and conduction strength. |
| `cover.rs` liquid kind: hydrocarbon if t_surface < 200 K, else water | Smooth | Each liquid's own phase diagram decides. A world could have both, or neither. |
| `ice_giant_mass_earth` (regimes.rs) | Edge | An envelope-composition question: ice-rich against H/He-rich. Comes from formation (F track). Keep it as a derived label. |
| `plates_min` 8 and `plates_max` 16 | Fudge | The plate count should follow from the convection wavelength and mantle depth. It stays until S2b tectonics; registered. |
| `tide.q` 100, `tide.k2` 0.3 for all bodies | Fudge | Tidal response depends on rheology (Maxwell time against forcing period) and on the interior layering. Use the interior model once it exists; until then give the ice and rock values separately. Registered. |
| `greenhouse_k` 30 | Fudge | Already registered. Replaced by the atmosphere stage. |
| `plates.continental_threshold_km` 25 and other crust thicknesses | Keep | These are geometry for the plate model, not regime gates; they can stay as tuned parameters. |

Not gates: Poisson sampling cut-overs (`lam < 30`), the cube-sphere geometry, and colour tables. Weather (`sse-weather`) and life have their own regime rules and are audited when their next package runs (W and E tracks); the same rule applies there.

## 3. What changes in the build plan

- **S0b** becomes "process strengths for regimes": replace the first-match `rules` list with the strength blocks in 2.1. It keeps `regime` as a derived label, adds the fixtures from the exogeology review, and changes golden hashes once, on purpose. It runs with S16a, as already accepted. Weather's W7a (one climate crate under erosion and cover, `SSE4-weather-review.md`) changes the surface goldens too, so W7a, S0b and S16a land together and surfaces move once.
- **S17 to S21** (volatile ices, ice-shell stresses, cryovolcanism, colour and tholins, small bodies, water release) are written as strength blocks from the start. M1's vapour-pressure block also serves the cold trap, land ice and shadow ice rows above.
- **Every other track** applies the rule when its next package touches a gate: weather regimes (W), life and biomes (E), stellar activity (O, granulation and Rossby-number spots and flares, 21 Nov), and classification (C, which becomes the reader of strengths).
- The citations thread checks each relation named here before it is relied on, as the standing rule requires.
