# S0b, part 1: icy bodies by process strength

Design: `handover/design/SSE4-processes-not-classes.md` (accepted 21 Nov, Alex: "do the new rules - LEAN INTO
THE PHYSICS!") and the exogeology review's queue (accepted 21 Nov: the three icy-body bugs first). This is
the first slice of S0b: the regime rules an icy world meets now ask physics questions. The rest of S0b
(lid mobility, heat-pipe fraction, melt fraction, ocean worlds from hypsometry) follows.

## The bugs it fixes
- Enceladus and Miranda were lumps: `small-body` was radius 300 km or less, for ice and rock alike.
- Callisto lost 96% of its craters and gained Europa's cracks: every water world was `ice-shell`, renewed
  by heat flow.
- Triton kept too many craters (497 listed) for a surface about 10 to 100 Myr old.

## What changed
- `sse-surface/src/lib.rs`, three new quantities a rule can read (QUANTITIES is now 16):
  - `round_ratio`: radius over the radius at which self-gravity overcomes material strength,
    R = sqrt(3σ / (2πGρ²)). σ is the geometric mean of ice (5.6 MPa) and rock (154 MPa), weighted by ice
    volume. These are calibrated so ice rounds at about 200 km and rock at about 300 km (Lineweaver and
    Norman 2010). This is a real edge (gravity against strength), so the rule keeps a cut at 1.
  - `shell_activity`: whether tidal cracks can cross the shell. The conductive shell thickness is
    d = 651 ln(273.15 / T_s) / q (ice conductivity 651/T). Strength = x⁴ / (1 + x⁴), with x = 15 km / d
    (Rudolph et al. 2022). It is a smooth curve; the regime label is read off it at 0.5.
  - `tidal_w_m2`: today's tidal heat plus `Body.tidal_history`, the new input for past spans such as a
    resonance passage or the years after a capture. Imports, packs and authors give it; formation will
    fill it when F makes captures. It is skipped in saves when empty.
- Base surface pack rules:
  - `small-body` now reads `round_ratio` at most 1 (was radius at most 300 km).
  - `ice-shell` needs `shell_activity` at least 0.5.
  - A new `cold-ice-lid` (water fraction at least 0.05) catches the rest. It is a solid ice surface with
    no renewal of its own and no landforms of its own (Moore et al. 2004, Callisto).
- `regimes.rs`, fossil landforms: what an earlier regime built (Ganymede's grooves from its resonance era)
  stays on the surface, dated in its own era.
  - A fossil survives the renewal of every later regime, exp(−∫renewal).
  - It also survives the craters that came after it only as a crater of its own width would. This is the
    same saturation law, e^(−N/N_sat), that thins old craters (`CraterField::erasure_survival`; Gault 1970,
    Hartmann 1984). A fossil from the heavy bombardment is cratered away; one from a quieter era is kept.
  - The cause says so: "Fossil: no later regime has renewed it, and the craters since have not erased it."
  - `RegimeInputs.craters` carries S4's crater field into the regime stage. Without it (sketch-only
    callers), only renewal applies.
- `grow` is the one landform builder, used for the current regime and each fossil era.
- Fixtures: Ganymede is now `cold-ice-lid`, with `tidal_history` from 1.5 to 2.5 Gyr at 0.08 W/m²
  (Showman and Malhotra 1997; Bland et al. 2009). Callisto, Enceladus, Miranda and Triton are added
  (Triton with authored tidal heat of 0.1 W/m², its capture heating).

## Results (base pack, fixtures)
| Body | Before | Now |
|---|---|---|
| Enceladus, Miranda | small body (lumpy) | round, cold ice lid |
| Vesta | lumpy | lumpy (rock, below the edge) |
| Callisto | ice shell, 46 of 1,178 listed craters kept, 515 cracks | cold ice lid, 1,138 kept, no cracks |
| Ganymede | ice shell | cold ice lid today; grooves and bands from its resonance era kept |
| Europa | ice shell | ice shell |
| Triton | 497 craters kept | ice shell, 1 kept |

## Checks
- `tests::gravity_rounds_ice_sooner_than_rock`, `tests::ice_shells_are_active_only_when_cracks_cross_them`.
- `impacts_tests::cold_ice_lids_keep_their_craters_and_young_shells_lose_them`.
- `regimes_tests::fossil_landforms_survive_later_quiet_but_not_heavy_bombardment`.

## Goldens re-blessed on purpose
- `surface.json` (Ganymede's regime and timeline, the new fixture bodies).
- `surface-impacts` (Ganymede now keeps 16 large craters, not 46).
- `contract.jsonl` (envelope lines carrying the new quantities).
The reason is this package, under the processes-not-classes rule.

## Known limits, next
- Ganymede's resonance era renews the whole surface, so only 16 craters of 105 km or more are kept. Real
  Ganymede kept its dark terrain, about a third of the surface. Renewal by area fraction comes with the
  ice-shell cracking rule (exogeology queue item 3): cracking strength from tidal stress against ice
  strength, eruptions from a melt supply, each over the part of the world it reaches.
- Enceladus is a cold lid globally. Its active south polar terrain needs the same rule, with heat
  concentrated at a pole.
- Triton's cantaloupe terrain is drawn as Europa-style chaos.
- The `heat` and `heat-now` landform activity, (q / q_ref)³, is still the old fudge (fudge register).
