# Fudge register

Alex, 19 Nov 2026: "the physics is WAY more interesting than faking it. But understand a browser has limited
capability. Fall on physics rather than simplification where practical. If we decide to "fudge" it is a known and
documented fudge with a damn fine reason to exist."

A fudge is any deliberate simplification, stand-in, tuned constant or placeholder that is not the published
physics. Each one is listed here, in the PR that adds it, by the thread that adds it.

Columns: **where** (file and item), **what it does instead of the physics**, **why** (the damn fine reason:
browser cost, missing data, not built yet, tuned to match an observation), **real physics that would replace it**,
**status** (stand-in until package X, permanent with reason, to review).

Add entries under your area. Keep one row per fudge.

## Stars

### Fudge register

| # | Where | Instead of the physics | Why (as the code or log says) | Real physics to replace it | Status |
|---|---|---|---|---|---|
| F-S1 | sse-atoms/src/star.rs:3-6, the atoms `ms_luminosity` and `ms_radius` | Metallicity-free empirical MLR and MRR (solar-neighbourhood average) | "a metallicity input would change nothing and pretend otherwise"; evolved stars need tracks | The HPT00 fits already in `sse-formation::hurley` (Z-dependent L_ZAMS, L(τ), R(τ)) could become atoms. The base-physics flow and the formation suite would then share one stellar model. | Known; documented in the atom text |
| F-S2 | sse-formation/src/stars.rs:39-46 | DK13 lower limits (≥ 50, ≥ 60, ≥ 80 percent) used as point values at chosen bin centres (3, 12, 30 M☉), with log-mass linear interpolation | not stated | Use the DK13 values as lower bounds, or Moe and Di Stefano 2017's multiplicity by mass and period (it also gives correlated period and mass-ratio distributions) | Undocumented; should be logged |
| F-S3 | sse-formation/src/stars.rs:157, :202 | Flat mass ratio q ~ U(0.1, 1) at all periods and all primary masses | "Flat mass ratio … (Raghavan 2010)" | Raghavan's q is roughly flat with an excess of twins at q ≈ 1. Moe and Di Stefano 2017 give q by period and mass (steeper at long periods, twin excess at short). | Partly documented |
| F-S4 | sse-formation/src/stars.rs:171 (8/44) and sse-cosmos stars.rs:104 (0.2) | Triple share of multiples: two suites, two values, quadruples ignored | "triples 8 of 44 percent" | Raghavan abstract (verified): 8 percent triple and 3 percent higher order, of 44 percent multiple. Use one pack value (about 0.25 if higher orders fold into triples) in both suites. | Inconsistent; undocumented |
| F-S5 | sse-formation/src/stars.rs:207-212 | Triple stability: outer semi-major axis ≥ 3.5 × inner apoapsis. The comment says "periastron". The outer eccentricity is drawn after the check, so the outer periastron can fall inside the limit. | "Mardling and Aarseth 2001, order of magnitude" | Mardling and Aarseth 2001: a_out/a_in > 2.8 (1 + q_out)^{2/5} (1 + e_out)^{2/5} (1 − e_out)^{−6/5} (with an inclination correction), applied after e_out is drawn | Documented as an order-of-magnitude choice; the comment does not match the code |
| F-S6 | sse-formation/src/lifeline.rs:142-143 | Pre-main-sequence duration = G M²/(R_ZAMS L_ZAMS) (31.4 Myr in solar units); constant L and R through the pre-main sequence | "Kelvin-Helmholtz time" | The KH time is usually G M²/(2RL), half the value used. Real PMS times come from tracks: about 30–50 Myr for 1 M☉ and hundreds of Myr for 0.1 M☉ (Baraffe et al. 2015). Tabulated PMS tracks (Baraffe 2015, MIST) would also give the bright early luminosity that matters for young planets. | Labelled "sketch" |
| F-S7 | lifeline.rs:101-104 `GIANT_FRACTION` = 0.1 | All post-HG phases (RGB, HB, AGB) last 0.1 t_MS | "HPT00 figure 4 gives about 0.06 … 0.1" (not checked) | HPT00 §5.2–5.4 give t_He and AGB lifetimes (eqs. 41-45 for the giant branch timescales, eq. 57 onward for helium burning). Port them as the next SSE stage. | Labelled sketch (F0 and T1 known limits) |
| F-S8 | lifeline.rs:105-107 `L_TIP_LOW_MASS` = 2500 L☉; 10 × L_BGB for high masses | RGB tip luminosity fixed; no HB or AGB | sketch | Tip from HPT00's core-mass-luminosity at M_c,HeF, then HB and AGB. The Sun's largest radius is 0.85 au against 1.0–1.2 au in detailed models (T1.md). | Labelled sketch |
| F-S9 | lifeline.rs:108-110 `M_UP_WD` = 8 | Fixed white dwarf / neutron star boundary | "8 solar masses is the usual observed limit" | HPT00 M_ec from the zpars (zp[5] = zp[4] + 1.8), metallicity-dependent | Documented |
| F-S10 | lifeline.rs:111-112, :161-167 | NS mass 1.4, black hole above 25 M☉ with mass max(0.25 M, 3) | "Sketch" | HPT00 eq. 92: M_NS = 1.17 + 0.09 M_c,SN, black hole if M_c,SN > 7 (read in the arXiv PDF). Or a modern prescription such as Fryer et al. 2012 rapid/delayed. | Labelled sketch |
| F-S11 | sse-cosmos/src/stars.rs:254-321 | A whole analytic stand-in stellar model for G5: L = M⁴ or 1.4 M^3.5, R = M^0.8, giant L = 30 L + 50, R = 20 R, WD L = 1e−3 t^−1.4, WD R = 0.0126, BH 0.3 M | "analytic stand-in for pack tracks" | Reuse `sse_formation::lifeline::LifeLine` (HPT00) or at least the Eker atoms (fix S-P3). The two suites then agree on what a star is at an age. | Documented as a stand-in; uncited |
| F-S12 | sse-cosmos stars.rs: IFMR Kalirai 2008, BH above 20; formation: Cummings 2018, BH above 25 | Two IFMRs and two black-hole thresholds for the same galaxy | not stated | One pack table used by both | Inconsistent |
| F-S13 | sse-formation/src/inherit.rs:87, :162 | Z = 0.02 × 10^[Fe/H] for the stellar fits, while the disc uses protosolar Z = 0.0142 (Lodders 2003, disc.rs:91) | "the fits' own solar scale" (F0.md) | Defensible: HPT's models reproduce the Sun at Z = 0.02, so the scale belongs to the fit. Keep it, but register the dual solar Z so nobody "fixes" one to match the other. | Documented in F0.md; not in a register |
| F-S14 | sse-time/src/star.rs:233-256 `blackbody_rgb` | Helland's curve fit (blog, non-refereed) to a blackbody colour table | "Helland's fit to the CIE blackbody locus" | Integrate the Planck function against the CIE 1931 colour-matching functions to XYZ, convert to linear sRGB, normalise and gamma-encode. It is cheap enough to tabulate once per 100 K. | Undocumented as a fudge |
| F-S15 | sse-time/src/star.rs:177-196 `core_at` | Non-clocked stars: core grows linearly from 0.4 × remnant mass to the remnant | "sketch"; `core_start_fraction` 0.4 in packs/base-time, no source | HPT00 core mass at the BGB (eq. 44 onward, M_c,BGB) and core growth (eqs. 34-39) | Labelled sketch; parameter uncited |
| F-S16 | lifeline.rs:266-270 | WD cooling A = 4 for CO white dwarfs where HPT00 says 15 | the comment claims it is HPT's value | See fix S-P1. Either the paper's 15, or keep 4 as a documented tuning toward detailed cooling models. Better still, tabulated cooling (Bédard et al. 2020). | Mis-cited (fix S-P1) |
| F-S17 | sse-time/src/binary.rs:43-53 `BinaryRules::default` | q_crit 3 for all main-sequence donors (HTP02 says 0.695 for low-mass MS, fix S-P6); α = 1, λ = 0.5 fixed; conservative transfer | "defaults from the papers"; F7.md says move them to packs/base-time | HTP02 and BSE use q_crit = 0.695 for deeply convective MS donors (M < 0.7), and λ from envelope structure (Claeys et al. 2014; Dewi and Tauris 2000). Non-conservative transfer with a β parameter. | Labelled sketch; numbers not in a pack yet |
| F-S18 | sse-formation/src/stars.rs:144 | Eccentricity 0 below 12 d, U(0, 0.8) above | "Tidal circularisation below about 12 days (Raghavan figure 14)" | Raghavan and Moe and Di Stefano give e distributions that widen with period (roughly flat to thermal) | Partly documented |
| F-S19 | stars.rs:136-140 | log P clamped to [−0.5, 10]; separations raised to the triple limit after 20 tries | not stated | A truncated log-normal with explicit bounds (Roche contact at the low end) | Undocumented |
| F-S20 | lifeline.rs:258-264 | Brown dwarf R fixed at 0.1 R☉; L from a single power law at all ages, floored at 1 Myr | "sketch" | Burrows 2001 or ATMO 2020 / Baraffe 2015 tables for L, R, Teff(t, M) | Labelled sketch |
| F-S21 | sse-atoms `ms_luminosity_at_age` | Gough's solar brightening applied to every mass by fraction of the main-sequence life | the atom says so | HPT00 L(τ) (eq. 12), already ported | Documented in the atom |
| F-S22 | sse-cosmos stars.rs:261 (`birth`) | Birth-cluster FUV G0 = 10 (M/1000)^0.5 | not stated (pack sources cite Fatuzzo and Adams 2008 for a median about 1000 G0) | Fatuzzo and Adams 2008 FUV distribution by cluster size. Note that this formula gives about 10 G0 for a 1000 M☉ cluster, two orders of magnitude below the cited median of about 1000. | Possible physics bug; outside stars proper, flag to the galaxy register |
| F-S23 | crates/sse-formation/src/lifeline.rs white dwarf cooling (decision on F-S16) | Mestel cooling with A = 4 for every white dwarf; HPT00 eq. 90 gives A = 15 for carbon-oxygen (4 helium, 17 oxygen-neon) | With 15, a 0.6 M☉ white dwarf at 1 Gyr is about 1e-4 L☉, several times fainter than detailed cooling models; 4 tracks them more closely | Tabulated cooling sequences (Bédard et al. 2020 Montreal tables) by mass and core composition | documented 19 Nov, to replace |
| F-S24 | crates/sse-time/src/star.rs CORE_LUM_K | Core mass-luminosity constant 2.3e5 fixed at Z = 0.02 | Simplicity | HPT00 eq. 37's Z-dependent D (already computed as fits.zpars[6]); 47% higher at Z = 0.001 | to fix |

### Candidates to move into flows

| Where | What it computes | Blocks it would need | View that would show it |
|---|---|---|---|
| sse-formation/src/hurley/mod.rs `l_zams`, `r_zams`, `t_bgb`, `t_ms`, `l_tms`, `r_tms`, `ms_state` | Z-dependent ZAMS L and R, main-sequence lifetime, L(t), R(t) | `star.zams_luminosity(M, Z)` and `star.zams_radius(M, Z)` (Tout 1996), `star.t_bgb(M, Z)` (HPT eq. 4), `star.t_ms` (eqs. 5-7), `star.tams_luminosity` and `tams_radius` (eqs. 8, 9), `star.ms_track(τ)` (eqs. 12-13), `teff` (Stefan-Boltzmann) | An HR diagram with the star's track from ZAMS to TAMS, a scrubbable age marker and the Sun's track for reference |
| sse-formation/src/lifeline.rs `LifeLine::new` (phase boundaries) | Pre-main sequence, MS, HG, giant and remnant dates | `star.kh_time`, `star.t_ms`, `star.t_bgb`, `star.giant_duration`, `star.remnant_kind(M)`, `star.ifmr(M)` (Cummings) | A life-line timeline bar (phases as coloured spans, log time) under the HR diagram |
| lifeline.rs `state_at` remnant branches | WD cooling (eq. 90), WD radius (eq. 91), NS cooling (eq. 93), Schwarzschild radius | `remnant.wd_luminosity(M, Z, t, A)` (A as a visible parameter), `remnant.wd_radius(M)`, `remnant.ns_luminosity`, `remnant.bh_radius` | A white dwarf cooling curve, L and Teff against cooling age, with colour bands |
| sse-time/src/star.rs core clock and Reimers wind | Core mass against L (eq. 37), core growth, wind mass loss | `giant.core_luminosity(Mc, Z)`, `giant.core_growth`, `wind.reimers(L, R, M, η)`, `orbit.adiabatic_widening(a, M)` | Giant-branch panel: L, R, M and Mc against time, plus planet orbits widening and the engulfment line |
| sse-time/src/star.rs `blackbody_rgb` | Star colour | `light.planck_xyz(T)` (CIE CMF integral), `light.xyz_to_srgb` | Colour swatch on the star card with a Teff slider |
| sse-time/src/binary.rs `roche_fraction`, `q_crit`, CE energy, conservative transfer | Binary interaction outcome | `binary.roche_lobe(q)` (Eggleton), `binary.q_crit(phase, Mc/M)` (HTP02), `binary.ce_final_separation(α, λ, …)` (energy formalism), `binary.conservative_a(Md, Ma)` | Binary panel: both stars' radii against time against their Roche lobes, the contact date and the outcome branch (stable, CE or merger) with numbers |
| sse-formation/src/stars.rs `kroupa_quantile`, `multiplicity_fraction`, `companion_orbit` | Drawn primary mass, companion count, periods, q, e | `imf.broken_power_law` (pack edges and slopes), `multiplicity.by_mass` (table), `binary.period_lognormal`, `binary.q_flat`, `binary.e_by_period`, `kepler.a_from_p` | Population view: the IMF histogram with the drawn star marked, and the companion period distribution with this system's companions |
| sse-cosmos/src/stars.rs `StarParams::state` | G5 stellar state | Reuse the HPT flows above (fix S-P3 / F-S11) rather than a separate model | Patch-view star tooltip with phase, L and Teff |
| sse-formation/src/inherit.rs Z mapping | [Fe/H] → Z for the fits | `metals.z_from_feh(feh, Z☉)` with Z☉ as a visible parameter (0.02 for HPT, 0.0142 for discs) | Inheritance card showing [Fe/H] → Z for the fits and Z for the disc |

## Orbits and formation

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Interiors and heat

### Fudge register

| Where | Instead of the physics | Why (as the code/log says) | Real physics to replace it | Status |
|---|---|---|---|---|
| crates/sse-atoms/src/heat.rs:220, `sse.heat.active_geology_lifetime` param `q_crit` = 0.019 W/m² | A fixed radiogenic heat-flow threshold for "geology stops" | "a calibration, not a law": half of Earth's present radiogenic flux, tuned to put Mars's widespread volcanism ending about 1 Gyr ago and the Moon's mare volcanism in its first 2 Gyr | Parameterised mantle convection (Nu–Ra scaling, stagnant-lid versus mobile-lid regime) with secular cooling: melt production when T_mantle exceeds the solidus at lid depth (e.g. Foley and Smye 2018; O'Rourke and Korenaga 2012) | documented, editable param |
| heat.rs:153, :226 (`radiogenic_heating`, lifetime) | Radiogenic heat only: no primordial (accretion and core) heat, no tides, no lid style | Stated in the science text | Add a secular-cooling term (Earth loses about as much again; KamLAND 2011) and tidal heating input (`tides.heating_power` exists) to the heat-flow sum | documented |
| heat.rs:71 (`radiogenic_heating`) | Every body's budget is BSE today wound back by the **Sun's** age 4.567 Gyr, then scaled by `rel`; body age is time since formation | Implicit: rel is "relative to the Sun's birth cloud" | Fine if `rel` is defined at the star's birth. Document that the body is assumed to form at its star's birth (Myr offsets ignored) | undocumented, harmless |
| heat.rs:37-52 isotope shares | Atom fractions used as mass fractions | Oversight | Mass fractions (fix P1/P2) | to fix |
| heat.rs:176 (`sse.heat.mantle_mass`) | All U, Th, K in silicate; core gets none; crust not separated | Lithophile partitioning (MS95) | Fine to first order; K in the core (up to a few hundred ppm, debated) and crust/mantle split for heat-flow profiles | documented |
| heat.rs:265-267 (`al26_planetesimal_heating` params e_dry_start 6e5, e_dry 1.8e6 J/kg, dry_residual 0.1) | Tuned energy thresholds and a log-linear water-loss ramp | Params with short descriptions, no source | Derive from c_p ≈ 800–1000 J/kg/K × (T_dehydration ≈ 600–900 K − T_0) for the start and (T_solidus ≈ 1400 K − T_0) + latent heat ≈ 4e5 J/kg for melting; residual water from melt solubility (e.g. Lichtenberg 2019 methods) | tuned, uncited |
| heat.rs:272 (`al26_planetesimal_heating`) | Adiabatic upper bound: all Al-26 energy stays in the body | "for a body larger than a few tens of kilometres" | Conductive cooling with radius: τ_diff = R²/κ against τ_Al = 1.03 Myr (a radius input and a loss factor 1 − f(R²/κτ)) | documented |
| heat.rs:115 Al-26 initial | Single canonical 26Al/27Al for all bodies, scaled by `al26_rel` | Canonical CAI value | Fine; note the debate over heterogeneity (lower in chondrule precursors) | documented in text |
| crates/sse-atoms/src/planet.rs:21-26 (`radius_from_mass`) | Chen and Kipping median relation, no scatter, no composition, no irradiation or age | "a forecast, not a composition model" | Composition-aware chain: rocky (Zeng 2016) + water mantle + H/He envelope (Lopez and Fortney 2014) for sub-giants; giant evolution tables (Fortney 2007 / Baraffe) for giants | documented |
| planet.rs:126-129 (`rocky_radius`) R⊕ unit | Uses equatorial R⊕ 6378.1 km in place of Zeng's 6371.0 km | Oversight | Fix P4 | to fix |
| crates/sse-formation/src/early.rs:149-151 (`rocky_radius`) | Zeng 2016 used outside 1–8 M⊕ | "a labelled extrapolation" | For M < 1 M⊕ use Seager et al. 2007 / Zeng 2016 table models; the 1/3.7 exponent overestimates compression in small bodies | documented |
| early.rs:212 and crates/sse-place/src/model.rs:197 | Water mantle as radius × (1 + 0.5 × water fraction) | "Zeng et al. 2019: 50 percent water, about 25 percent larger" | Zeng et al. 2016/2019 tabulated rock-water curves or a two-layer EOS; the linear factor is a fit to one point | documented-ish, reference incomplete |
| early.rs:207 | FeO oxidation: a water-rich body loses up to half its core, `ox = min(water/(water+rockmetal), 0.5)` | "first order ... (sketch)" | Redox equilibrium Fe + H2O → FeO + H2 with a water budget limit | sketch |
| early.rs:179 (`GIANT_GAS_FRACTION` 0.5), crates/sse-formation/src/violent.rs:260 (giants 11.2 R⊕) | Fixed giant radius for encounters; gas fraction cut for "giant" | "11.2 for giants (as F6)" | `radius_from_mass` (CK17) or giant cooling tracks | stand-in |
| early.rs:156-160, model.rs:198-202 (Lopez and Fortney envelope) | Passes core or solid mass where LF14 uses total mass; comment says "solar metallicity, enhanced opacity" with age exponent −0.18 | Not explained | Re-read LF14 eq. 3: total mass, and pick one opacity case (−0.11 solar, −0.18 enhanced) | inconsistent, to check |
| crates/sse-place/src/lib.rs:344 | Shoreline escape velocity with a fixed CMF 0.33 and Zeng radius at the upper mass prior | Not explained (quick bound) | Use the drawn body's CMF and the `rocky_radius` atom | stand-in |
| crates/sse-atoms/src/layers.rs:35 (brine `osmotic_coefficient` 0.93) | One constant osmotic coefficient for every salinity and temperature | "non-ideality of the brine"; good to about 10 percent below 20 percent salt | Pitzer model for NaCl–H2O (φ(m, T)), or tabulated freezing points (CRC), ending at the 252 K eutectic | documented |
| layers.rs:67 (ice shell) | Conduction only, k = 651/T, no convection, no porosity, no salt or clathrate | "No convection: thick shells convect" | Stagnant-lid convection threshold (Ra_crit) and Nu scaling for thick shells (e.g. McKinnon 1999), porosity-reduced k near the surface | documented |
| layers.rs:116 (hydrostatic) | Constant g and ρ | "fine for oceans and crusts" | Integrate dP/dr = −ρ g(r) with g(r) from enclosed mass (Adams–Williamson with an EOS) | documented |
| crates/sse-import-v3/src/lib.rs:509-510 | Missing V3 ocean depth taken as Earth's mean 3688 m | "a missing depth is taken as Earth's mean" | Fine as a default; cite (Charette and Smith 2010) and flag the value as assumed in evidence | uncited default |

### Candidates to move into flows

| Where | What it computes | Blocks it would need | View that would show it |
|---|---|---|---|
| crates/sse-atoms/src/heat.rs:61-78 `radiogenic_heating` (one atom, four isotopes inside) | Sum over 4 isotopes of c·share·e^{λ·4.567}·rel·h·e^{−λt} | `sse.heat.isotope_heating` (one isotope: concentration, share, H, half-life, rel, age; cited per isotope) ×4 + a `sum` block; the budget params become pack data per isotope | Stacked-area chart of W/kg against time (0 to 15 Gyr) per isotope, with "now" marked: shows U-235/K-40 early and Th late |
| heat.rs:213-251 `active_geology_lifetime` (bisection inside an atom, recomputing the whole heat chain) | Time when the radiogenic flux falls to q_crit | Reuse the planet-heat chain (heating → power → flow) as a function of age, plus a generic "time where a curve crosses a threshold" solver block (fixed 80 bisections, deterministic) | Heat flow against age curve with the q_crit line and the crossing marked, and Earth, Mars and Moon overlaid |
| heat.rs:253-292 `al26_planetesimal_heating` (energy and water loss in one atom) | Remaining Al-26 energy after formation time; water kept | Split: `sse.heat.al26_energy` (H0 from Al wt%, ratio, W/kg: each a cited param) → `sse.planetesimal.devolatilisation` (energy to water kept, with thresholds derived from c_p·ΔT + latent heat blocks) | Energy against formation time (log) with dehydration and melting bands; slider for `al26_rel` |
| crates/sse-place/src/model.rs:186-208 `radius_re` and crates/sse-formation/src/early.rs:149-160, :212, violent.rs:258-272 (three Rust copies of the radius chain) | Radius from mass and make-up: CK17 for giants, Zeng rock × water factor + LF14 envelope | Existing `sse.interior.rocky_radius`; new `sse.interior.water_mantle_inflation`, `sse.envelope.lopez_fortney_thickness`, and a `select` block (gas fraction ≥ 0.5 → `sse.planet.radius_from_mass`). One flow `planet.radius` used by place and formation | Mass-radius diagram (log M against R) with Zeng iron/rock/water curves, CK17 median line and the body as a point, with its layer stack (core, mantle, water, envelope) to scale |
| crates/sse-formation/src/early.rs:162-245 (XUV fluence and energy-limited envelope loss) | Envelope survives if absorbed XUV exceeds binding energy | `sse.star.xuv_fraction` (Jackson 2012 saturation and decay), `sse.integrate.log_time` (fixed 200-step), `sse.envelope.binding_energy`, `sse.envelope.energy_limited_loss` (efficiency param, Owen and Wu 2017) | XUV luminosity against time with the cumulative absorbed energy against the binding energy line |
| crates/sse-world/src/lib.rs:574-630 `physics_notes` (Rust glue chaining brine freezing → ice lid heat flow) | Is each brine liquid; heat flow needed to hold the ice lid over it | A `layer.ocean-lid` flow: salinity → `sse.ocean.brine_freezing_point` → base temperature → `sse.heat.ice_shell_heat_flow` (+ `sse.interior.hydrostatic_pressure` for the ocean floor pressure), evaluated per stack pair | The Strata ladder with each shell annotated (freezing point, liquid or frozen, W/m² through the lid), plus a temperature-depth profile through lid and ocean |
| crates/sse-place/src/lib.rs:339-350 (shoreline escape-velocity bound with fixed CMF 0.33) | Escape velocity of the heaviest allowed rocky world | `sse.interior.rocky_radius` → `sse.planet.escape_velocity` (both exist) | Escape velocity against mass with the shoreline cut |
| crates/sse-import-v3/src/lib.rs:501-580 `hydrosphere` | Surface water in Earth oceans | `sse.ocean.surface_water_volume` (coverage × 4πR² × depth) + a normalise-by-Earth block | Globe coverage with an "Earth oceans" gauge |

## Surfaces

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Climate, weather and life

### Fudge register

| Where | Instead of the physics | Why (as the code or log says) | Real physics to replace it | Status |
|---|---|---|---|---|
| F1. `feedback.rs:183-187`, `history.rs:523-527` | O₂ burial is 0.1% of each metabolism's **energy cap**, and nothing on O₂ pushes back except √O₂ weathering | L2.md:36: "nothing in the box yet raises oxidative weathering or fires to match" | Feed realised production (see fixes P3). Burial efficiency falls with O₂ through phosphorus retention (Van Cappellen and Ingall 1996). Fire caps land biomass above about 25% O₂ (Lenton and Watson 2000; Belcher and McElwain 2008). Use the COPSE functional forms (Bergman et al. 2004). | **Undocumented error, not a fudge.** It explains the 51% O₂: land adds 42 Gt C/yr to an ocean already counted at its 118 Gt light cap instead of its 38 Gt realised production. No misremembered constant is needed to explain it. |
| F2. `feedback.rs:149-152`, pack:725-727 | Reducing gases decline as 3e12 × exp((4570 − t)/2000 Myr) | "until the planet layer's interior models exist" | Outgassing from the S-track heat budget × gas redox from mantle fO₂ (Holland 2002 gas composition; Kump and Barley 2007 subaerial fraction from continental growth) | Documented stand-in. The source wording overclaims (fixes M3). |
| F3. `feedback.rs:205-209`, pack:728 | Oxidative weathering = 6.5e12 √(O₂/PAL), organic only, no pyrite cycle | Holland 1978; Bolton et al. 2006 as cited | The S cycle: pyrite burial is 7.8 Tmol O₂/yr, about 40% of modern O₂ sources (Catling and Claire 2005). Kerogen weathering is erosion-limited above a few % PAL (Bolton 2006, memory). | Partly documented; exponent unverified |
| F4. `feedback.rs:169-175` | Oxic and anoxic chemistry blended by 1/(1 + f/1e-5); CH₄ lifetime interpolated between 9.6 (9.1) and 1000 yr | Avoids flip-flopping; a fixed point found by bisection | CH₄ lifetime against O₂ from photochemical models (Pavlov et al. 2000; Goldblatt et al. 2006 parameterise the ozone-shielded CH₄ lifetime against O₂) | Documented in L2.md |
| F5. pack:730 vs pack:685 (`decay_methane_oxic`) | 0.25% of production decays as CH₄ in box H; 0.5% in box I | Reeburgh 2007 cited for 0.5% | One value from a methane budget: natural emissions about 200 to 250 Tg CH₄/yr against NPP (memory) | **Inconsistent**, undocumented |
| F6. `feedback.rs:259-264` | CO₂ index = 1/land_weathering_factor (0.25) as soon as land life exists | Berner 1997 | Silicate-weathering steady state, CO₂ against T with the weathering kernel (Walker, Hays and Kasting 1981; GEOCARB). A ×4 rate gives much less than a ×4 drop in CO₂ once temperature feeds back. | Documented as a stand-in; the mapping is undocumented |
| F7. `signatures.rs:191-197` | O₂ = 0.21 × min(production/1e17, 1) | L1d: a sketch calibrated to Earth today; L2.md says box I does not read box H yet | Read O₂ and CH₄ from box H's air (`AirBox.series`) | Documented |
| F8. `signatures.rs:103`, pack:682 | CH₄ ppm per g C/yr 4.5e-15 (fit to 1.9 ppm) | Calibrated to one point (L1d.md) | Steady state τ/(12 N_air) from box H (4.2e-15 for 9.1 yr) | Documented |
| F9. `signatures.rs:245-246` | Red edge at 700 √(5772/Teff) nm | Kiang 2007 cited | Kiang et al. 2007 II: pigment absorption is predicted from the photon-flux peak and the top-of-canopy spectrum. No √ law. | **Undocumented invented scaling** |
| F10. pack:686-695 | Likelihood ratios 20, 3, 1.5, 2, 10, 0.5; detection floors | Bayesian framing (Meadows 2018; Krissansen-Totton 2018) | Detectability from retrieval S/N (instrument model); the LRs stay judgment calls | Documented as choices |
| F11. `biosphere.rs:132-134, 326-343` | Flyer 250 kg (ρ/1.225)^1.5 (g/g⊕)^−3.5; land animal 7e4 kg × g^−3; plant 130 m/g | L1c: rule table | Flight: required power ∝ (W³/ρ)^½ against muscle power ∝ mass (Pennycuick). Land: bone stress ∝ g M^⅓. Trees: hydraulic limit (Koch 2004) depends on g through the water-column potential, about 0.01 MPa per m × g. | **Uncited exponents** |
| F12. `biosphere.rs:698-715` | lineages = 0.05 area^0.25 √provinces √bins × grade | "a sketch" (L1c.md) | Species-area z ≈ 0.25 is a real empirical exponent (Arrhenius 1921; MacArthur and Wilson). The 0.05 is tuned. | Documented sketch |
| F13. pack:458, 504-507, 561, 580 | Origin rate 0.0018/Myr, innovation rates, population exponent 0.25 | "tuned so Earth's inputs give an Earth-like outline… one data point, not a law" | None possible. It is the honest unknown, with dials. | Documented, with a damn fine reason |
| F14. pack:313, `budget.rs:199-206` | Radiation cap 1000 Sv/yr flat | Daly 2009 | Dose-rate tolerance by organism, plus a repair rate | Documented |
| F15. `budget.rs:263-267` | Missing light peak defaults to 500 nm, band to 400-700 nm | Listed in `missing` | Star Teff through the atmosphere's transmission | Documented (a gap is listed, not hidden) |
| F16. `budget.rs:362-364` | Volume habitats use V^(2/3) as their area for feeding levels | none | Production per volume, with a volumetric viability threshold | Undocumented |
| F17. budget pack:316-330 | P reuse 1000/100/46/10 and C:P 1000 on land | Benitez-Nelson 2000; Cleveland 2013 | Explicit P cycle: weathering supply → recycling → burial | Documented, partly unverified |
| F18. `climate.rs:66-87` and `planet-climate.json` | Grey radiative equilibrium, τ input, no convection | The atom's science text says it overstates thick atmospheres | Radiative-convective adjustment (a lapse-rate cap): T_s from τ at the radiating level with a dry or moist adiabat below (Robinson and Catling 2012/2014 analytic RC model) | Documented in the atom |
| F19. `climate.rs:44-50` | Redistribution f is a free parameter (0.25 in the flow) | — | f from the day-night heat-transport efficiency (Koll and Abbot 2016 scaling, against surface pressure and rotation) | Documented |
| F20. planet-climate flow, test | Earth τ = 0.83 gives 286.7 K against the 288 test, at the edge of tolerance | — | τ 0.87 for T_eq 254 (fixes M5) | Minor, numeric |
| F21. `build-base-atlas.py:232-233` | "Filter mask" profile allows CO₂ up to 0.05 bar | Narrative category | No mask removes ambient CO₂. A rebreather or CO₂ scrubber is a sealed system. | Undocumented fiction. Label it as a game convention. |
| F22. `build-base-atlas.py:226-235` | Human gravity band 0.3 to 1.5 g, temperature 263 to 313 K | none | No long-term data exist for gravity; temperature from WBGT and heat-stress limits (wet-bulb 35 °C; Sherwood and Huber 2010) | Uncited |
| F23. `packs/base-time/pack.json:27` | Ocean loss at 1.1 S⊕ (the runaway value is 1.06 elsewhere in `early.rs:248`) | Wolf and Toon 2015 | One HZ block with the Kopparapu 2013/2014 polynomial S_eff(Teff) = S₀ + aT + bT² + cT³ + dT⁴ per limit and per mass | **Two different runaway numbers in two crates** |
| F24. `feedback.rs:300-303` | Oxygenated = O₂ ≥ 1% for 200 Myr | Avoids counting whiffs | (a definition) | Documented |

### Candidates to move into flows (small cited physics blocks)

| Where | What it computes | Blocks it would need | View that would show it |
|---|---|---|---|
| `budget.rs:150-167, 260-279` (light energy cap) | Carbon fixed from starlight in a band | `sse.light.band_fraction(peak, lo, hi)` (Planck); `sse.life.photosynthetic_yield(flux, share, efficiency, area)`; `sse.unit.energy_to_carbon(J per g C)` | A spectrum strip: the star's Planck curve, the pigment band shaded, the efficiency arrow, then g C/yr |
| `budget.rs:336-354` (Liebig) | min(energy cap, P supply × reuse × C:P) | `sse.life.nutrient_cap(p_new, reuse, c_per_p)`; `sse.math.min_with_label` (says which cap won) | Two bars per habitat, energy against nutrients, with the limiting one highlighted (this also exposes the P3 bug) |
| `budget.rs:362-374` (feeding levels) | log(density/min)/log(1/efficiency) | `sse.life.trophic_levels(density, min, transfer)` | A trophic pyramid with each level's g C/m²/yr |
| `feedback.rs:166-218` (the air box, one step) | O₂ balance: burial + escape − reductants − weathering (implicit √) | `sse.redox.organic_burial(prod, fraction)`; `sse.redox.reductant_flux(t)` (later from interior); `sse.atmosphere.diffusion_limited_escape(f_T, area)` (Catling and Claire eq. 4); `sse.redox.oxidative_weathering(O₂, PAL, k, exponent)`; `sse.solve.reservoir_step(implicit)` | A stacked source/sink ledger chart over time (the `Ledger` already exists): burial, escape, reductants and weathering, with O₂ and CH₄ lines. This is the view where the 51% shows up. |
| `feedback.rs:172-181` (methane steady state) | CH₄ = flux/(1/τ + k_esc) | `sse.atmosphere.steady_state_mixing(flux, lifetime, sinks…)`; `sse.atmosphere.ch4_lifetime(O₂)` (Prather oxic, Pavlov anoxic) | CH₄ against O₂ phase plot with the bistable branches (Goldblatt 2006) |
| `feedback.rs:259-264` (CO₂ index) | CO₂ drawdown by land | `sse.carbon.silicate_weathering(T, CO₂, plant factor)`; `sse.climate.grey_surface_temperature` (exists); a `solve` node for the steady state | A CO₂-temperature thermostat curve with the plant factor slider |
| `signatures.rs:191-210` | O₂ and CH₄ seen from afar | Read box H's outputs (`atmosphere.o2_fraction`, `atmosphere.ch4_ppm`) instead of the sketch; `sse.observe.detectable(value, floor)` | The observer's card: gas bars against detection floors |
| `signatures.rs:245-246` | Red-edge wavelength | `sse.light.photon_peak(Teff)` (3669.7 μm K, textbook); a pigment-edge rule from Kiang 2007 II | Same spectrum strip as the light cap, with the edge marked |
| `biosphere.rs:326-343` (flight, size, plant height) | Size limits from g and ρ | `sse.bio.max_flyer(ρ, g)`; `sse.bio.max_walker(g)`; `sse.bio.max_tree_height(g)`, each with its scaling cited | A silhouette scale panel against Earth's |
| `biosphere.rs:380` (pace of life) | Arrhenius factor | `sse.bio.arrhenius_rate(E, T, T_ref)` | Pace dial against temperature |
| `early.rs:245-250`, base-time `runaway_greenhouse_flux` | HZ limits | `sse.climate.hz_seff(Teff, limit, mass)` (Kopparapu polynomials); `sse.climate.insolation` (exists) | An HZ band diagram (Teff against S_eff) with each planet plotted; both crates then use one block |
| `climate.rs` (already blocks) | — | Add `sse.atmosphere.jeans_flux(λ, n, v₀)` next to `jeans_parameter`; `sse.climate.radiative_convective_surface` | Escape chart: λ for H, He, H₂O, N₂ and CO₂ against exobase temperature |

### The 51% oxygen gap

No misremembered constant explains it. The constants that set the steady state check out against Catling and Claire 2005, which I read: reductants 3e12, an O₂ inventory of 3.8e19 mol, and a burial share of about 0.1% of NPP.

The cause is **F1 (fixes P3)**. The air box is fed each metabolism's energy cap rather than its realised production:
- The ocean's oxygenic input is 118 Gt C/yr (the light cap) instead of 38 Gt C/yr (nutrient-limited production). Earth's 21% before land therefore comes from an error.
- Land adds its real 42 Gt C/yr on top, giving 160 Gt C/yr: burial 1.33e13, minus reductants 3.1e12, then (1.02e13/6.5e12)² × 0.2095 = **0.518**.

Two smaller physics errors (CH₄ lifetime 9.6 → 9.1 yr; escape coefficient ×2 high) do not affect the oxic steady state.

## Galaxy and cosmology

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Light, rendering and sound

Rule from the owner: fall back on physics where practical. A fudge has to be known, documented, and have a good reason to exist.

### 1. Fudges

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|
| `ui-contract/src/light.ts:327-373` `describeLight` (stars) | A star is a pure blackbody at Teff: no absorption lines, line blanketing, limb darkening, chromosphere or corona. | The kernel publishes no light facts yet. A blackbody gets the colour right to a few per cent for FGK stars. | A model-atmosphere spectrum per star (a results-as-data grid such as PHOENIX or ATLAS, by Teff, log g and [Fe/H]) as a kernel light-description block. Corona and activity come from design 4A (O10). | documented ("Stubbed" in the why text and the O1 log) |
| `light.ts:355-366` (planets, reflected light) | Grey reflectance: the star's spectrum times one Bond albedo. Full phase is assumed, and the dilution factor is wrong (/4; see fixes 2.1). | There is no reflectance spectrum or phase angle in the scene facts. | A wavelength-dependent reflectance from the surface and atmosphere (Rayleigh scattering, cloud and haze, surface minerals, ice, vegetation red edge), a phase function from the observer geometry, and geometric albedo p(λ). | documented as stubbed; **the factor is a bug, not a fudge** |
| `light.ts:367-369` (planet heat) | Thermal emission at one `surface_k`, with emissivity 1 − Bond albedo. | There is no temperature map or atmosphere emission yet. | Emission from the atmosphere's emitting layers (brightness temperature by band; CO₂ and H₂O bands), with day and night sides. | documented as stubbed |
| `light.ts:407-420` `teffFromSpectral` | A hand table of class temperatures (O 41000 … L 2100 K), linear in subclass, used for players' scenes. Luminosity class is ignored, so a K1 III giant gets a dwarf's Teff. | Players are told a spectral type, not Teff. | A citable table (e.g. Pecaut and Mamajek 2013, from memory and to be verified) shipped as pack data, by luminosity class. Better still, a disclosed Teff fact from the kernel. | labelled "Stubbed", no source; **needs a source** |
| `light.ts:255,287` camera white | The camera is white-balanced to a 6504 K Planckian, not the CIE D65 spectrum. | A blackbody needs no table, and D65's correlated temperature is 6504 K. | The tabulated CIE D65 spectral power distribution (300–830 nm), since D65 sits off the Planckian locus. The difference is a slight tint. | in a code comment; not in the recipe legend |
| `light.recipes.json:14-16` Camera recipe | Box-car RGB filters (400–490, 490–580, 590–700 nm) and saturation ×1.6. | It stands for "a camera and a press stretch", and the legend says so (`actual-enhanced`). | Measured sensor or filter response curves (for example a named camera or the HST/JWST filter set) as pack data. | documented in the recipe legend |
| `light.ts:61` `NARROW_M` | Every narrowband filter is a 3 nm box centred on one line. [S II] at 671.6 nm loses most of its 673.1 nm partner. | It keeps one width for all lines. | Per-filter centre and width (or transmission curves) as recipe data. Treat [S II] as a doublet. | in a comment ("roughly"); the doublet is not mentioned |
| `light.ts:252-283` Draper cut | Any purely thermal source below 798 K is shown as no colour in visible recipes. | Chromaticity normalises brightness away, so without the cut a 300 K world would show as deep red. | A visibility test on absolute brightness: visible-band luminance at the observer against the eye's threshold (photopic and scotopic), after distance. This needs the observer geometry that O9 sensing brings. | documented (comment and O1 log); good reason |
| `light.ts:161-166` `chromaticity` | Colour only: the brightest channel is set to 1 and out-of-gamut channels are clipped at 0. | Brightness spans many decades, which a mark cannot show. | Exposure and tone mapping with a stated exposure (design 2.3), and gamut mapping rather than clipping. | documented in the doc comment |
| `light.ts:262-276` single-band floor | Light more than 30 decades below the brightest visible light is dropped, and the ramp spans the recipe's `decades`. | This is a display range. | Instrument sensitivity and noise floor from the sensor model (design 5A / O9). | documented in a comment |
| `light.ts:65-75` lines | No body has emission lines yet (`lines: []`), so the SHO and HOO recipes show only continuum. | Nebulae are not built yet (O3, O4). | A line-strength grid from Cloudy and MAPPINGS (O4). | documented in the recipe `why` |
| `ui-contract/src/compare.ts:97-115` `starColour` | A hand-set colour ramp by Teff (7 stops, 2500–30000 K) for star discs on the Sizes tab. | It is quick, and "presentation only". | Use `thermalColour(teff)` from light.ts (the same CIE chain as the Eye recipe), so a star has one colour everywhere. Today the Sizes tab and the Eye view can disagree. | documented as "rough"; **cheap to replace** |
| `reference-objects.json:23` brown dwarf | A "typical" 66000 km and a hand colour `#8a4b3a`. | No single real brown dwarf stands for all of them. Old brown dwarfs are about Jupiter-sized whatever their mass. | Radius from an evolution grid (Burrows et al. 1997/2001 or Sonora) at a stated mass and age. Colour from a cool-atmosphere spectrum, not a blackbody (molecular absorption dominates). | `typical: true`; the radius claim is unverified |
| `reference-objects.json:32` neutron star | One pulsar (PSR J0030+0451, 12.71 km) stands for "typical", with a hand colour `#c9d6ff`. | It is a measured, citable value near the middle of the NICER range. | Keep the measured value, and say "one measured pulsar" in the type. The colour could come from `thermalColour` of its surface temperature (about 10⁶ K gives blue-white), which agrees. | `typical: true`; documented |
| `reference-objects.json` stellar Teffs | Teff values rounded (Arcturus 4290, Sirius B 25000) or from another work than the one cited (Sirius A, α Cen B, Proxima). | Written from memory. | Use the cited value or cite the Teff separately (see fixes). | to fix |
| `compare.ts:11-13` | "× Earth" and "× Jupiter" use mean radii (6371.0, 69911), while IAU nominal radii are equatorial (6378.1, 71492). | They match the fact-sheet radii of the objects. | No change of physics; state "mean radius" in the size words or tooltip. | undocumented in UI |
| V14 measure (`handover/packages/V14.md:19`) | Distances between bodies assume circular, coplanar orbits at the ruler's time. | The chart draws orbits that way. | Kernel ephemeris positions (Kepler elements with e, i, Ω, ω) at the ruler's epoch. | documented in the panel text |
| `ui-contract/src/sound.ts:85,200` | Kepler's third law in AU and M☉ is multiplied by the Julian year (365.25 d) rather than the Gaussian year (365.2569 d). The planet mass is left out of M. | It is simple, and the 7 ppm error is inaudible. | Use the kernel's orbital period fact (already derived by the orbit flows), and fall back only when it is absent. | undocumented; trivial |
| `sound.ts:145-150` octave bands; `:109` scale | Pitch bands by mass and a just-intonation "settled" dial. | A designed sound, labelled so in the Why text and in the honesty label. | None (a mapping, not physics). It stays a documented design choice. | documented |
| O1 test `light.test.ts:20-26` | Expected colours from Mitchell Charity's table, within ±10/255. | A published reference for blackbody sRGB. | Keep, but verify the table values (unverified this session), or compute them from the CIE 1931 tabulated functions rather than the Wyman fit. | values unverified |

### 2. Candidates to move into flows (physics chain blocks)

The owner wants the physics in chain blocks (except big blocks) with a UI element that shows it. Each row below is something the TS presentation layer computes today that is physics and could be a block.

| Where | What it computes | Blocks it would need | View that would show it |
|---|---|---|---|
| `light.ts` `describeLight` (stars) | A star's spectrum (now a blackbody at Teff, weight 1). | `star.teff_k`, `star.radius_sun` (exist) → **`light.stellar-spectrum`** (Planck now; grid lookup later, results-as-data) → `light.description` fact (continuum components and lines). | A **spectrum chart** in the body card: log-λ axis, continuum curves, band strips, line ticks, the recipe's channels drawn over it, plus the "Why?" chain. The Eye and band recipes on the system chart read the same fact. |
| `light.ts` `describeLight` (planets) | Reflected starlight plus own heat. | `planet.bond_albedo`, `orbit.a`, star spectrum, `surface_k` → **`light.reflected`** (dilution (R*/a)², phase function, geometric albedo) + **`light.thermal-emission`** (emissivity, brightness temperature) → `light.description`. | The same spectrum chart, with the reflected and thermal parts as two stacked curves and the crossover wavelength marked. This shows directly why planets "come closer to their star" in mid-IR. |
| `light.ts` `xyz`, `linearSrgb`, `thermalColour` | Colour of a spectrum to the eye. | Colour is presentation, so keep it in TS. But a **`light.apparent-colour`** block (CIE XYZ, chromaticity x, y) is worth having as a fact so that Minerva's text tier and the players' facts can say "orange-white" from the same number. | A chromaticity swatch with a "CIE x, y" readout in the body card. A small CIE diagram with the Planckian locus would show where each star sits. |
| `light.ts` `teffFromSpectral` | Teff from a spectral-type string for players' scenes. | **`star.teff-from-spectral-type`** (pack table by class and luminosity class), run on disclosed facts only. | The players' chart colours and the players' body card ("Teff ≈ 5100 K, from type K1 V, estimated"). |
| `light.ts` Draper cut | Whether a thermal source is visibly glowing. | **`light.visible-glow`**: visible-band luminance at the surface (and later at an observer) against the photopic threshold. | An Eye recipe legend note ("too cool to glow"), and lava worlds or brown dwarfs on the chart glowing for a reason. |
| `compare.ts` `starColour` | Star disc colour on the Sizes tab. | Reuse the `light.apparent-colour` fact, or `thermalColour(teff)` directly. | The Sizes tab discs, matching the Eye recipe exactly. |
| `sound.ts` `orbitScene` period | Orbital period from a and M when no year fact exists. | Already a kernel quantity (orbit flows). Read `orbit.period_s` and stop recomputing in TS. | The sound "Why?" panel names the period fact and its chain. |
| V14 measure | Distance between two bodies now, and min/max for co-orbiting bodies. | An **`orbit.position-at-epoch`** (ephemeris) block, then a separation (light time = d/c). | The Measure panel on the chart, with a "Why?" naming the ephemeris block and its assumptions. |
| `reference-objects.json` brown dwarf radius | A typical radius. | **`substellar.radius-from-mass-age`** (evolution grid, results-as-data), already needed by system formation. | The Sizes tab: "typical brown dwarf, 40 M_J at 5 Gyr" with the radius computed, not typed. |
| light.recipes.json `why` texts (Wien, Rayleigh–Jeans) | Prose physics claims. | None (prose). The spectrum chart could draw λ_peak = b/T and mark the R–J tail, so the claim is visible. | Spectrum chart annotations. |

**UI element to visualise the chain.** One **Spectrum** panel per body (and per star) does most of this. It is a log-wavelength plot of the light description from its blocks, with the fixed band set as shaded strips, emission lines as ticks, and the active band recipe's channels overlaid. Under it are the block chain (stellar spectrum → reflected + thermal → description) and each block's "Why?". It shows what every recipe is integrating, makes stubbed parts visible (dashed curves labelled "stubbed"), and is the natural home for O2 and O4 evidence (brightness temperatures, line ratios).
| `packages/renderer/src/atmosphere.ts` (R2a) | Air optics (Rayleigh per gas, scale height) computed in the renderer | The kernel op did not exist yet | Kernel `read_air_optics` (light track, PR 24) | Stand-in until PR 24 merges; then the renderer reads the op |
| `atmosphere.ts` gas table | One refractivity per gas near 550 nm, no dispersion | Small table, colour carried by the lambda^-4 law | Per-wavelength refractivity (Peck and Reeder, Bucholtz tables) | To review with the kernel op |
| `gpu-air.ts`, `canvas.ts` | No aerosols (Mie haze) and no absorption (ozone, methane) | Not built yet | Mie scattering from the weather op's aerosols; absorption bands from the light track | Stand-in until R2b |
| `gpu-air.ts` | Single scattering only | Exact for thin air, no lookup tables to build | Hillaire (2020) multiple-scattering LUT; thick air (Venus, Titan) is too dark without it | Stand-in |
| `gpu-globe.ts` physical lighting | Ground hue from the survey palette, scaled to the kernel's albedo | No spectral reflectance per material yet | Spectral reflectance per material (light track) | Stand-in |
| `passes.ts` relief pass | The star's direction is a look setting (azimuth, elevation) and its light is white | The view has no ephemeris position at the ruler's date yet | Star direction from the orbit at the campaign date; star colour from its spectrum | Stand-in |
| `canvas.ts` air | One-column, plane-parallel air along view and sun; no limb glow beyond the disk | Canvas tier is the weak-device path | The GPU ray-march (already on GPU tiers) | Permanent for the canvas tier (weak devices do less) |

## Interface and player views

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Candidates to move into flows

Logic written in Rust that could be an editable, cited flow of small blocks (Alex, 19 Nov: "try and do everything in
the physics chain blocks... other than Big Blocks. And have a UI element that makes sense to visualise").

| Where | What it computes | Why it is not a flow yet | Blocks it would need | View that would show it |
|---|---|---|---|---|
