# RK1: rocket data on the master data sheet (track RK)

Atomic Rockets plan (`handover/design/SSE4-atomic-rockets-plan.md`, sections 6 and 9; accepted by Alex 23 Nov 2026). The site is used only as a map to primary sources. Nothing here cites it, copies from it, or is named after it.

Sources gathered on 23 Nov, with quotes and the gaps: `RK1-data/RK1-sources.md`. Every value is marked unverified. They were read from fetched text, and several of the reports are OCR scans, so each needs a page-image check.

## Part 1: propellants and the impulse unit (this PR)

Built by `scripts/propellant_rows.py`, which is imported by `scripts/build-base-reference.py`.

- **New substances:**
  - `n2h4` hydrazine, `mmh` monomethylhydrazine, `udmh` 1,1-dimethylhydrazine, `n2o4` dinitrogen tetroxide, `h2o2` hydrogen peroxide.
  - Each has `phases.boiling_point_k` (with `at_pa`), `phases.melting_point_k` (`triple_point_k`/`_pa` for N2O4) and `phases.density_liquid` (with `at_k`).
  - All but H2O2 have `chemical.states.liquid.dhf`. The liquid enthalpy of formation for H2O2 was not found.
  - Molar masses are worked out from the formulas, never typed.
- **Added to existing rows:**
  - `boiling_point_k` for ch4, o2, xe and ar.
  - `critical_point_k` and `critical_point_pa` for xe, kr and ar, the gases electric thrusters store supercritical.
- **New mixture `rp-1`:** density near 23 C from NISTIR 6646. The surrogate formula C11.8H23.0 is in the note.
- **New unit `isp_s`:** the second of specific impulse, a velocity, with factor g0. "465 s" means 465 × g0 m/s.

Disagreements kept in the notes:
- MMH boiling point: 360.1 K (NASA RP-1013) against 364 K (NIST).
- UDMH boiling point: 337.1 K (NIST) against 336.5 K (CHRIS).

## Part 2: nuclear reactions (next)

D-T, D-D (both branches) and D-He3 Q values are worked out from the sheet's AME2020 masses. The neutron's share comes from two-body kinematics. Fitzpatrick's 17.59 MeV, with 14.05 MeV for the neutron, is the cross-check for D-T.
- p-B11 needs the B-11 mass. The AME2020 file would not fetch.
- U-235 fission energy split: Sher and Beck 1981 (EPRI NP-1771).
- The `reactions` table checks that elements are conserved, so nuclear rows need a `kind: "nuclear"` that checks nucleons and charge instead. That is a change to the sheet reader (sse-reference).

## Part 3: materials and shields (after)

- **Found:** melting points for W, Mo, Nb, Ta and Re (Lundberg 1985, LANL); C-103 and Inconel 718 hot strengths (manufacturers' sheets); UO2 (ORNL/TM-2000/351); UC and UC2 (NASA MSFC); NIST SRD 126 densities and hydrogen fractions for water, polyethylene, W and Pb.
- **Still needed:** graphite sublimation, carbon-carbon, ZrC, NbC, HfC, Ti-6Al-4V, Al 6061-T6, 304 stainless, LiH, borated polyethylene. Next sources: NASA SNP-HDBK-0008 and PNNL-15870.
