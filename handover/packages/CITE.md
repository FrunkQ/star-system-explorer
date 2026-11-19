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
  white dwarfs come out several times fainter than detailed cooling models). Comment corrected; fudge F-S23.

### Orbits and formation (19 Nov)
Physics changed, goldens re-blessed on purpose (time, time-generated, formation-growth, formation-upheaval,
formation-leftovers):
- Giant-star tide (`sse-time/src/epoch.rs`): the efficiency factor was inverted, min(1, (τ/2P)²); Villaver and
  Livio 2009 use (P/2τ)² capped at 1. Tides on planets round giants are now up to 1e2 to 1e4 times stronger: in
  the Solar System run Venus is engulfed (as Schroeder and Smith 2008 find) and Earth survives about 3 percent
  inside pure widening; T1 test updated. A planet dragged inside the star is floored at half its radius so the
  engulfment check catches it (the faster decay otherwise reached a = 0 and NaN).
- Pebble isolation mass (`sse-formation/src/growth.rs`): Bitsch et al. 2018 eq. 26's diffusion term
  α/(2 St)/λ added (about +42 percent at α 0.01, St 0.1). F3 population tests still pass.
- Reference text: Kelvin-Helmholtz coefficient and type I reduction now say what the code uses; Marcus 2009 for
  the iron fraction; Obertas fit range; Jackson 2012 X-ray saturation by spectral type; Lodders vs Asplund Z.

### Surfaces (19 Nov)
Wording only, no physics value changed; goldens re-blessed because reference strings are hashed
(surface-impacts: REFS; surface-regimes: Venus corona cause text):
- Crater saturation 0.15 D^-2 is about 10 percent of geometric saturation (top of Gault 1970's 1 to 10 percent), not
  "a few percent (Hartmann 1984)"; kept as a documented fudge.
- Giant cloud chemistry thresholds are on 1-bar temperature, adapted from Sudarsky et al. 2000's Teff classes
  (clear class not modelled); documented fudge.
- Lourenço 2020 DOI corrected (G-cubed); Europa's 60 Myr surface age credited to Schenk et al. 2004; Venus coronae
  "several hundred, 60 to over 2000 km" (Stofan 1992 gives no count of 500); Neukum chronology valid to ~4.1 Gyr.

### Galaxy and cosmology (19 Nov)
Physics changed, goldens re-blessed on purpose (haloes, galaxy, envelope, stars, cosmos, cosmology; galaxy-events after merging G6 from main, and the galaxy_chart cases in contract.jsonl after merging the Explore tab, since both follow the merger histories):
- Major-merger mass ratio (`sse-cosmos/src/haloes.rs`): drawn from the full Fakhouri, Ma and Boylan-Kolchin 2010
  eq. 1 shape, ξ^β exp((ξ/ξ_t)^γ), by a fixed 40-step bisection on its integral. The old draw kept only ξ^β and
  called the exponential "about 1"; it rises about 3 times from ξ = 0.25 to 1, so mergers now lean towards equal
  masses. Every downstream galaxy history moves.
- Disc heating exponent (`packs/base-cosmos` velocity.power): 0.35 → 0.40, the total-dispersion exponent of
  Holmberg, Nordström and Andersen 2009 section 6.3 (U 0.39, V 0.40, W 0.53). Encounter rates move.
- Hawking lifetime (`sse-cosmos/src/epochs.rs`): 2.1e67 → 1.14e67 (M/M☉)³ yr. Page 1976's total power for
  massless species, counting only the photon and graviton shares (19 percent) because neutrinos have mass far
  above the Hawking temperature of any stellar hole. The dark-era date moves about 0.27 dex earlier.
- Reionisation "why" text now quotes Planck 2018's with-BAO column (7.8 ± 0.7) as well.
- Metadata only: Correa et al. 2015 is paper I (MNRAS 450, 1514) eqs. 14, 15 and 20 to 23; Sheth and Tormen eq.
  10 and its ν convention; Muratov 2015 wording; Planck z*; Asplund protosolar Z; spin width in ln, not dex;
  Johnson and Li 2012's radius-dependent floor stated beside the single −1.0 value.

### Runaway greenhouse limit (19 Nov)
Physics changed; goldens re-blessed on purpose (time, time-generated: event text only):
- `sse-formation/src/early.rs`: the runaway limit was 1.06 times Earth's insolation, Kopparapu et al. 2013's
  section 3.1 value, which their 2014 letter (ApJL 787, L29) corrected after an error in the water infrared
  coefficients. It now follows the 2014 table 1 by planet mass: 0.99 at 0.1, 1.107 at 1 and 1.188 at 5 Earth
  masses, straight in log mass between them and flat outside. No body in the formation-early golden sits between
  the old and new limits, so that golden does not move.
- `packs/base-time` keeps 1.1 as the ocean-loss point and now cites the 2014 value; the T1 event text cites 2014.
