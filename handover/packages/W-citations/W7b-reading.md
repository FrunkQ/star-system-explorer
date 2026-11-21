# W7b reading notes (21 Nov 2026)

Read by a research pass via WebFetch of the arXiv full texts (paper hosts block direct downloads from the build
container, so a summarising fetch transcribed the equations; algebra and sample numbers re-checked where noted).
DOIs marked "(from memory)" are still to be checked against the publisher.

- **Mitchell and Vallis 2010**, JGR 115, E12008, doi 10.1029/2010JE003587 (from memory); arXiv 1008.1996, read in full.
  Eq. 3.3b: Ro_T = U/(2Ωa) = R T0 Δ_H / (2Ωa)². Superrotation "occurs for values of Ro_T exceeding one"; the
  transition run is Ro_T = 1.3. Runs at 0.02 (Earth-like), 1.3, 10.5. Quoted values: Earth 0.02, Mars about 0.1,
  Titan about 30, Venus about 1200. **No scaling of the superrotation strength with Ro_T is given** (figure 2 shows
  only the three cases), so the shape of our 0-to-1 strength is our own (docs/fudges.md).
- **Lindzen and Farrell 1980**, J. Atmos. Sci. 37, 1648, doi 10.1175/1520-0469(1980)037<1648:ASARFT>2.0.CO;2.
  σ_max ≈ 0.31 f (dU/dz) / N. The paper's own PDF is a scan; the 0.31 (0.3098) is confirmed only by a citing source
  (NCAR NCL `eady_growth_rate`). Abstract confirms growth linear in the surface temperature gradient. **Unverified
  from the paper.**
- **Komacek and Showman 2016**, ApJ 821, 16, doi 10.3847/0004-637X/821/1/16 (from memory); arXiv 1601.00069, read in
  full. Eq. 31 (A = ΔT/ΔT_eq, Δln p = ln(p_bot/p)): drag branch A ~ [1 + τ_rad τ_drag Δln p / τ_wave²]⁻¹; Coriolis
  branch A ~ [1 + τ_rad Δln p / (f τ_wave²)]⁻¹; drag applies when τ_drag < 1/f, Coriolis when 1/f < τ_drag (off the
  equator). τ_wave = L/(N H). τ_rad is prescribed in their model (the c_p p / (g σ T³) form is Koll and Abbot's).
- **Koll and Abbot 2016**, ApJ 825, 99, doi 10.3847/0004-637X/825/2/99 (from memory); arXiv 1605.01066 (some equations
  rebuilt by the fetch, unconfirmed). τ_rad = c_p p_s / (g σ T_eq³); τ_wave/τ_drag = C_D a / H.
- **Haqq-Misra et al. 2018**, ApJ 852, 67, doi 10.3847/1538-4357/aa9f1f (from memory); arXiv 1710.00435 v4, read in
  full. Eq. 8 λ_R² = sqrt(gH)/(2β); Eq. 9 L_R = π sqrt(U/β); β = 2Ω/a. Slow: λ_R/a > 1 and L_R/a > 1; Rhines:
  λ_R/a > 1, L_R/a < 1; fast: both < 1. (The abstract swaps the two for the Rhines regime; the body is followed.)
- **Wordsworth 2015**, ApJ 806, 180, doi 10.1088/0004-637X/806/2/180 (from memory); arXiv 1412.5575, read in full.
  Eq. 27 thin-radiator night temperature T_n ≈ [(1−A) F κ p_s / (4 σ g)]^¼, κ = 5e-5 m²/kg (a lower limit for thin
  air; 70.4 K at 0.01 bar reproduced). Collapse when the night side falls below the main gas's condensation
  temperature; comparing T_n with T_sat(p_s) by Clausius-Clapeyron is our construction. Eq. 43 CO₂ fit for p_crit
  (0.14 bar at 1366 W/m², reproduced), not used.
- **Knudsen edge**: λ = k T / (√2 π d² p) against the scale height (exobase at Kn ≈ 1); textbook (Chapman and
  Cowling), no DOI-bearing source checked.
