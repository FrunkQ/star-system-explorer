# Status: Master data sheet

Owner thread: "Build the master matter data sheet". Covers: MP data waves, MP22, SOL, DM1, DM6, citation checks.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`. Stage order: `handover/design/SSE4-coming-together.md`.

| Package | State | Log |
|---|---|---|
| MP24 where each element goes (element inventory part 1): per element Lodders 2003 condensation temperature, host and the formation material it rides in; CI chondrite share; Earth's core, silicate Earth and crust concentrations; water's heat of fusion; the Sun's rotation | done (data; all unverified reproductions, flags in the log); MP7 is the kernel's, MP6 and MP8 proven core's; scatter_dex rows next | packages/MP24-element-affinity.md |
| FT1 base-tech pack and strength for weight (track FT): citation check of the design note (its section 14); strength for weight on the sheet as tensile_strength and density on mixtures steel-wire, aramid, pbo-fibre, carbon-nanotube; then `packs/base-tech` with the limits, cited marks and the neutral ladder | done: parts 0 to 4 (`packs/base-tech`: 10 limits, 18 figures, 12 advances, the neutral ladder, part families, 5 balance checks, 13 cited real craft); the FT2 readers are the kernel thread's | packages/FT1.md |
| MP18 water's ices, the data wave (`scripts/water_phases.py` into `substances[h2o].phases`: 16 stable phases from vapour to superionic ice XVIII, 12 metastable, 36 boundaries graded measured, computed, extrapolated or convention, 21 triple points, the `phase_at` table 4401 × 1501 log p by log T, run-length encoded; IAPWS 2011 and 1992 check values reproduce; superionic and ionic edges computed or recalled and marked) | done (data); MP17 reader is the kernel thread's | packages/MP18.md |
| MP23 ship fuels (`fuel-*` mixtures by mass for base-nav fuel ids; liquid densities at the boiling point for h2, o2, d2, he3; isotope masses usable in formulas; no antimatter row on purpose) | done; helium-3 density unverified | packages/MP23-fuels.md |
| MP22 typed copies, ledger and check: every constant and material number typed outside the sheet was labelled with the row it should read, then moved | **done 23 Nov 2026: typed copies left 0** (kernel's last batch PR 326). Closed: `scripts/audit-matter.py --closed` in check.sh fails on any typed copy, with no ledger to label one; `handover/packages/MP0-typed-copies.txt` stays as an empty stub only because CI's audit step names it (switching CI to `--closed` and deleting the stub needs Alex's yes) | packages/MP22.md |
| SOL2 Sol's surfaces (average surface temperature, surface pressure, main gas for Mercury to Pluto where NSSDCA prints them; Earth's oceans 1 by the unit's definition; keys the surface chain reads first) | done | packages/SOL2.md |
| SOL1 the catalogued Solar System (`packs/base-sol` from `scripts/build-base-sol.py`): the Sun, eight planets, the Moon, Ceres and Pluto with mass, radius, orbit, rotation, obliquity and albedos in kernel field names, the belts and the Oort cloud as graded regions, every value cited (NSSDCA, Dawn papers, Connelly 2012); checked against the sheet's Sun and Earth | done; the chart room's "Open the Solar System" door reads it | packages/SOL1.md |
| DM1 the Cosmos group on the data sheet (galaxy histories design 3.1 to 3.3): H0, Ωb, Ωc, Ων, Ωm worked out, σ8, n_s, T_cmb, N_eff, ΩΛ worked out, w0 and wa, the dark matter "what it is" rows; the fourth mark "theoretical" with its note; page groups with the particle-route notes; the cosmology reads them through the one constants handle (`base-cosmology` keeps only `now_gyr`); the cosmos chain's stages in the reach report | done; the mark on the Reference screen is the chart room's; DM2 to DM5 read the rest | packages/DM1.md |
| DM6 constants with a history (galaxy histories design 3.2): a `law` on any measured constants row (constant, steady drift or a step at a date), never real (marked authored or fictional, with its own source), never on a worked-out row; `Reference::constant_on`, `has_history` and `handle_on(t_gyr)`, bit-identical to `handle()` with no laws; Kepler check with a fictional drifting G | done; dated kernel calls and the star's ESTIMATED mark along its life are the kernel's, the history graph on the Reference screen the chart room's | packages/DM6.md |
| SOL3 air and rings for the catalogued Solar System: `atmosphere.composition.<substance id>` (mole fraction, unit 1) for Earth, Venus, Mars, Pluto and the four giants (1 bar level), gas by gas from the NSSDCA fact sheets; the giants' main rings as regions with `kind` "ring", hosted by their planet (Saturn D, C, B, Cassini Division, A; Jupiter main; Uranus epsilon; Neptune Le Verrier and Adams) with `optical_depth` and `albedo` (range middles, ranges kept) from the NSSDCA ring sheets; the door enters them as `body.kind` ring, `ring.optical_depth`, `body.albedo` | done (Saturn's F ring left out: no width printed) | scripts/build-base-sol.py, scripts/sol_extra.py |
| RK1 rocket data on the sheet (track RK, Atomic Rockets plan section 6): part 1 propellants (n2h4, mmh, udmh, n2o4, h2o2 with boiling, melting, liquid density, liquid enthalpy of formation; boiling points for ch4, o2, xe, ar; critical points for xe, kr, ar; rp-1 mixture; unit `isp_s`), part 2 nuclear reactions, part 3 materials and shields | parts 1 (PR 348), 2 (PR 354, nuclear reactions from the sheet's masses, U-235 fission split) and 3 (13 materials: refractory metals, C-103, Inconel 718, UO2/UC/UC2 fuels, polyethylene, lead) done; gaps (graphite, carbon-carbon, carbides, Ti-6Al-4V, Al 6061, 304, LiH, borated PE) wait on a readable SNP-HDBK-0008 and PNNL-15870; RK2 next | packages/RK1.md |
| RK2 base-tech additions (Atomic Rockets plan question 3): heat figure `working_k` under limit 7, limit 11 beams spread, `least_size` on advances, flown advances (Hall, arcjet, ion, RTG; IKAROS on the light sail); kernel: `for` ceilings skipped, matching part first | done (this PR); solar arrays wait on an array mass | packages/RK2.md |
| MP25 protosolar isotope shares (`isotopes[].abundance_protosolar`, Lodders 2021 Table 4) for MP8c | done (PR 365) for the 12 stable rows H to O; Be, B, O-17, F, Ne wait for their isotope rows | packages/MP25-isotope-shares.md |
| MP26 solid propellants (`pban-ap-al`, `htpb-ap-al` with make-up, density, theoretical Isp) and RP-1 net heating value, for RS's real engines | done (PR 372); solids' heats of combustion not found | packages/MP26-solid-propellants.md |
| MP27 heat, melt and boil values for hull skins (`heat_capacity`, `heat_to_melt`, `latent_fusion`, `heat_to_boil`, `boiling_point_k`, `latent_vaporisation`; graphite `latent_sublimation`) on Al, Ti, Fe, Nb, Mo, Ta, W, Pb and graphite, for RK12 burn-through | done (PR 381); tungsten's boiling point, graphite's sublimation temperature, rhenium and steel heats still to find | packages/MP27-hull-skin-heats.md |
| RK10 one Bands table on the sheet (Atomic Rockets plan question 14): `bands` table (view, light, channel rows), reader checks, `SHEET_BANDS` in ui-contract | done (PR 383); light, renderer and chart room threads repoint their own strings | packages/RK10.md |
| MP28 MP27's gaps: tungsten boiling point (JANAF W(g), extrapolated 157 K), graphite sublimation point and heat (C1 to C5 vapour), `stainless-304l` (Kim 1975), `ti-6al-4v` melting point, `al-6061` heat capacity, rhenium heat of sublimation | done (PR 407); solids' heats of combustion, alloy latent heats and rhenium melting heats still to find | packages/MP28-skin-gaps.md |
| MP29 the rest of the stable isotopes H to Ne (Be-9, B-10, B-11, O-17, F-19, Ne-20 to 22) with masses, so every Lodders 2021 share has a row | done (PR 419); masses from NIST's pages (AME2016), AME2020 itself unreachable | packages/MP29-light-isotopes.md |
| MP30 electrical conductivity for MF2's field ledger: `earth-core-liquid` and liquid iron (Pozzo 2012), `giant-envelope` sigma profile along Jupiter's adiabat (French 2012), `seawater` with PSS-78 fits | done (PR 419); ionic water's 1e4 S/m still recalled | packages/MP30-conductivity.md |
| MP31 units for the units engine (ED1a): 16 working units, 10 imperial units (group `imperial`), source NIST SP 811; reader knows `area` and `volume` | done (PR 419) | packages/MP31-units.md |
| MP32 sea liquids: liquid water's absorption spectrum 380 to 2000 nm (Pope and Fry 1997, Hale and Querry 1973) and refractive index (Segelstein 1981); `saturated_liquid` tables (density, viscosity, surface tension, vapour pressure) for water, methane, ethane, ammonia, nitrogen, helium, CO2, argon, propane, each column citing its reference correlation, read through CoolProp; new rows `c2h6`, `c3h8` | done (PR 461); liquid methane and ethane absorption still unsourced; ethane as a volatile (vapour fit, colour) next | packages/MP32-sea-liquids.md |
| MP33 condensate optics, part 1: `titan-tholin` n,k 264 to 2422 nm (Khare 1984), shape `optical.refractive_index {lambda_nm, n, k}` for the giant-bands and plumes threads | part 1 done (PR 461); h2so4, nh4sh, chromophore and the MP34 ices wait (list in the log) | packages/MP33-condensate-optics.md |
| MP34 surface materials, part 1: Mars dust simulants `jsc-mars-1`, `mms-2`, `mgs-1` with k from 250 to 2000 nm (Martikainen 2023); troilite FeS n,k on two axes (Kranhold 2026); visible `optical.n_solid` for CH4 ice (1.32) and N2 ice (1.23) | part 1 done (PR 475); SO2, sulphur, CO2 frost, ice k, hematite, regolith next | packages/MP33-condensate-optics.md |


## Next, in order (as of 24 Nov 2026, for whoever picks this track up)

1. **MP32 follow-ups for the climate thread.**
   - **Liquid methane and ethane absorption.** Leads: Ramaprasad et al. 1978 (Icarus 35, 400), and Grundy et al. 2002 (Icarus 155, 486; it includes liquid methane at 93 K).
   - **Ethane as a volatile.** Needs Fray and Schmitt 2009's fit, a condensed colour and a latent heat. This goes through the golden lane, since it may move hashes.
   
   Details are in packages/MP32-sea-liquids.md.
2. **MP33 and MP34 condensate optics.** The renderer's giant-bands and plumes threads read these through Look up; the order is in packages/MP33-condensate-optics.md:
   - `h2so4` (from the Palmer and Williams 1975 page image);
   - `nh4sh`;
   - `jupiter-chromophore`;
   - SO2 frost;
   - sulphur allotropes;
   - N2 and CH4 ice;
   - CO2 frost.
3. **Still unsourced, and searched without luck so far:**
   - heats of combustion for the solid propellants (JPL TM 33-649, JPL 79-29 and Thiokol TWR-13009 searched);
   - latent heats for Ti-6Al-4V and Al 6061;
   - rhenium's heat of melting;
   - the solar-metallicity end of the element `scatter_dex` (Bensby 2014 Table C.3 at CDS was not read).
   
   The Cayrel 2004 metal-poor scatter values are written down in packages/scatter-dex-notes.md and are still to be put on the rows.
4. **Waiting on the readers of new rows.**
   - Proven core reads MP30's conductivities for MF2, through these fields:
     - `earth-core-liquid.sigma_elec` and `sigma_elec_icb`;
     - `iron.sigma_elec_liquid`;
     - `giant-envelope.sigma_elec_profile` and `metallic_from`;
     - `seawater.sigma_elec`, `sigma_elec_25c` and the PSS-78 fits.
   - The editing thread strikes the MP31 units off its WAITING list in figures.test.ts. Imperial units are in group `imperial`.
5. **Later:** p-B11 fusion (the B-11 mass is now on the sheet); OV12 naming at stage 5.

Working notes for this track:
- After any change to the sheet, or to scripts/build-ui-reference.py, merge main and re-run `python3 scripts/build-base-reference.py`, `python3 scripts/build-ui-reference.py` and `python3 scripts/build-rust-reference.py` just before merging. CI's ui job checks the generated copy, and on 24 Nov PR 419 merged a stale one because PR 410 changed the script while 419's CI ran, which turned main red until PR 422 and PR 445.
- From the cloud, janaf.nist.gov, the AME2020 hosts, webbook.nist.gov, omlc.org and srd.nist.gov all refuse curl. A web-reading agent reaches janaf, physics.nist.gov and omlc.
- Values read from fetched text are marked `unverified` until the citation check reads the paper.
