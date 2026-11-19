# CITE: citation and constant check (track started 19 Nov 2026)

Goal: every reference, published relation and quoted constant checked against its source; registers in
`docs/citations.md` (per area under `docs/citations/`) and `docs/fudges.md`. Thread "Verify physics citations".

Method: sources read through web search and fetch (curl to doi.org, crossref, arXiv and ADS is blocked in the
cloud container). A shared fetch limit of about 100 pages an hour cut the first pass short; rows not read are
marked unverified, never guessed. A second pass follows.

## Log

### Interiors and heat (19 Nov)
Physics changed, goldens re-blessed on purpose:
- K-40 share of potassium 1.17e-4 → 1.19e-4 (the old value was the atom fraction; the code needs mass fraction;
  Ruedas 2017 table 1, consistent with Turcotte and Schubert's 3.48e-9 W/kg for K).
- U-238 and U-235 shares 0.9928/0.0072 → 0.9929/0.00711 (mass fractions, same source).
- Al-26 decay heat 0.355 → 0.3583 W/kg (Ruedas 2017 table 2); CAI-time energy 5.26 → 5.31 MJ/kg (test updated).
- Zeng et al. 2016 rocky radius and its inverse use the paper's R⊕ = 6.371e6 m, not the equatorial 6.3781e6 m
  (atoms `sse.interior.rocky_radius`, `core_mass_fraction`, and the sse-place shoreline cut).
- Atom versions to 1.0.1: `sse.heat.radiogenic_heating`, `radiogenic_power` (text), `active_geology_lifetime`,
  `al26_planetesimal_heating`, `sse.interior.rocky_radius`, `core_mass_fraction`.
- Goldens moved: formation-pipeline (heat power, geology-end ages), surface, surface-impacts, surface-plates,
  surface-regimes, surface-wear (envelopes follow the heat), time-generated (ages), place (Zeng radius),
  contract.jsonl (formation_ran lines). Bulk silicate Earth heat +0.3%, rocky radii −0.11%.

### Light, Sizes tab and design files (19 Nov)
UI only, no golden moved:
- Sizes tab reference objects: Alpha Centauri B 597700 → 600500 km and 5180 → 5231 K (Kervella 2017, 0.8632 R☉);
  Sirius A 9940 → 9900 K and Sirius B 25000 → 25370 K (the cited papers' values); Ceres cites Park et al. 2019
  Icarus (the old DOI did not resolve); Titan 2574.8 km after JPL; planets point at their own NSSDCA sheets.
- `ui-contract/src/light.ts`: reflected light weight was (R*/a)²/4 (a whole-sphere absorbed-flux average);
  the disc radiance of a Lambert sphere at full phase is (2/3)·A·(R*/a)², so reflected light rises 8/3 against
  thermal emission in band views. Planck test reference corrected to 2.6239e13 with a 1e-4 tolerance.
- Design files: titles, years and DOIs fixed in the sound, weather and nebulae designs (repo and project copies).

### Climate and life (19 Nov)
Physics changed, golden re-blessed on purpose (life-feedback.json only):
- Methane lifetime in oxic air 9.6 → 9.1 yr (Prather, Holmes and Hsu 2012: 9.1 ± 0.9 yr total atmospheric).
- Hydrogen escape 1.3e16 → 6.7e15 mol O2 a year per unit CH4 mixing ratio: Catling and Claire 2005 eq. 4 gives
  1.34e16 mol H2 a year for f_CH4 = 1, and two H2 lost leave one O2. Earth's oxygen end state 0.60 → 0.58 on the
  first fixture; methane slightly lower in oxic air.
- Wording: Holland 2002 and Kump and Barley 2007 argue gas composition and the submarine-to-subaerial shift, not
  interior cooling; Stevenson 2017 (not 2015) for a_w 0.585; Kiang 2007 part II for the red edge.
Found, not yet changed: the air box (`feedback.rs`) is fed each metabolism's energy cap rather than realised,
phosphorus-limited production (`budget.rs` by_metabolism), which is why land-life worlds reach about 51% O2.

### Stars (19 Nov)
Physics changed, goldens re-blessed on purpose (stars, formation, formation-pipeline, contract.jsonl):
- G5 (`sse-cosmos/src/stars.rs` and `packs/base-cosmology`): multiplicity by primary mass now Duchêne and Kraus 2013
  table 1 (was 0.27, 0.46, 0.7, 0.9 at 0.5, 1, 10, 150 M☉); main-sequence L and R from the Eker 2018 and
  Demircan and Kahraman 1991 atoms instead of L = M^4 and R = M^0.8 (red dwarfs were up to 11 times too faint);
  triple share 0.2 → 0.25.
- Formation suite: Raghavan 2010 higher-order multiples (3 of 44 percent) folded into triples since systems cap at
  three stars (8/44 → 11/44).
- `sse-time/src/binary.rs`: q_crit 0.695 for main-sequence donors under 0.7 M☉ (Hurley, Tout and Pols 2002
  section 2.6.1); no golden moved.
- White dwarf cooling keeps A = 4 for all white dwarfs as a documented fudge (HPT00 eq. 90 gives 15 for CO; with 15
  white dwarfs come out several times fainter than detailed cooling models). Comment corrected; F-S16.
