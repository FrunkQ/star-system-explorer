# SSE4 visual-polish design note: citation check

Date of check: 24 November 2026. Covers the 18 "from memory" items in `SSE4-visual-polish-design.md`.

## How to read this

**I did not have the design note itself**, only the 18-item request list. For each item I tested the claim as worded in that list, plus specific numbers I supplied myself as test values. So a "wrong" below means "this number is wrong if the note uses it", not "the note says this". Compare each figure against the note before editing.

**Method.** Every DOI in this document was checked against the Crossref publisher record for journal, volume, pages and title. Six Consensus research threads (63 papers read in full by Consensus), then four follow-up passes using Consensus search, the Consensus full-text reader, and the open copies of papers (arXiv, author pages, NASA, spec sources). Three numbers I re-checked directly myself are marked "re-read by me".

**Evidence grades** used in the tables:

| Grade | Meaning |
|---|---|
| Primary | Read in the paper's own text or table |
| Abstract | Read in the paper's own abstract |
| Secondary | A later peer-reviewed paper or reference page restating it |
| Snippet | Seen only in a web-search summary; needs a human look before it is printed |
| Derived | My own arithmetic or derivation, not a sourced statement |
| Not found | Could not be located in anything readable |

**Suggested status in the note:** move an item to "checked" where the status column says Checked; keep "from memory" on the specific sub-claims listed under "Still open" for that item.

## Corrections to make in the note

Each applies only if the note says the thing in the first half of the sentence.

1. **Jupiter lightning (16).** Jupiter's flash rate per unit area is not similar to Earth's. Juno gives about 6.1 × 10⁻² flashes km⁻² yr⁻¹; Earth is about 2.8, roughly 47 times higher. "Similar to Earth" in the Juno papers refers to peak rates inside storms (up to four per second).
2. **Vega temperatures (5).** 10,150 K pole / 7,900 K equator is Aufdenberg et al. 2006, not Peterson et al. 2006 (which gives 9,988 / 7,557 K). The current best values are Monnier et al. 2012: 10,070 / 8,910 K, rotating at 77% of critical, not 93%.
3. **Rhines scale (6).** π·√(2U/β) is the width of one jet, not the spacing between eastward jets. It matches Jupiter's and Saturn's bands within a factor of 2 only if U is the local mid-latitude eddy speed (10–40 m/s), not the 100 m/s jet speed.
4. **B-ring edge (7).** The Mimas 2:1 distortion is not a fixed "tens of km"; it varies from 4 to 71 km over about 5.4 years.
5. **A-ring edge (7).** The Janus 7:6 seven-lobed pattern (12.8 km) exists only while Janus is on the inner leg of its orbit swap (2006–2010) and disappears afterwards.
6. **Density-wave wavelength (7).** λ = 4π²Gσ₀r_L / [D_L (r − r_L)]. The r_L in the numerator is easy to drop.
7. **Daphnis edge-wave height (7).** "Up to 1.5 km" is a NASA caption, not the paper (which says about 1 km), and a later caption reports almost 4 km.
8. **Unlit side of the rings (8).** "C ring and Cassini Division bright" holds only at low Sun and viewer elevations. At 20°/30° the A ring is at the transmission peak and the C ring at about half of it.
9. **Eclipsed Moon brightness (10).** The supported range is about 10⁴ to a few 10⁵ times fainter than the full Moon. No source was found for 10⁶.
10. **Galilean moons in eclipse (10).** Tsumura et al. give 10⁻⁶ to 10⁻⁷ of normal brightness, and call haze forward-scattering only "the most plausible candidate".
11. **Mars dust (11).** Single-scattering albedo in the blue (about 440 nm) is about 0.77, not 0.6–0.7 (those are ultraviolet values). Lemmon et al. 2015 give a size range of about 1 to 2 µm, not 1.4–1.8 µm.
12. **Methane rainbow (12).** 49° is right for n ≈ 1.29, but the radius runs from about 47° to 52° with temperature and wavelength. The secondary bow falls inside the primary for methane (derived, not sourced).
13. **Comets (13).** Womack, Sarid & Wierzchos 2017 is titled "CO and Other Volatiles in Distantly Active Comets" in print. Do not present the Haser C₂ scale length as lifetime × outflow speed; the two do not agree.
14. **Bennu (14).** The spin acceleration (3.63 ± 0.52) × 10⁻⁶ deg day⁻² is Hergenrother et al. 2019; Nolan et al. 2019 gives 2.64 ± 1.05. Bennu's surface-failure spin period is 3.6 h; 3.5 h is Ryugu's.
15. **Aurora (15).** Pink-to-purple with altitude and a strong H-alpha line is the Saturn result (Cassini). The Galileo Jupiter papers report no colour. On Venus the green line was reported as nightglow (Slanger 2001); "aurora" is the later reading (Gray 2014, 2025). On Mars, Gérard 2020 is dayglow, not aurora.
16. **Mars green aurora (15).** Published brightness is 101 ± 14 R (the 93 ± 13 R figure is from the earlier conference abstract).
17. **Curl-noise authors (17).** The second author is Jim Hourihan. "Houriham" is a typo in the ACM/Crossref record.
18. **GPU reproducibility (18).** "IEEE basic operations, no FMA contraction, no built-in transcendentals, fixed order" is necessary but not sufficient. Division and square root are also not exact in the shader specs, and rounding mode and subnormals must be pinned, which WebGPU's shading language cannot do.

## Status by item

| # | Topic | Status | Best evidence |
|---|---|---|---|
| 1 | Cox & Munk slope variance | Checked | Secondary ×2 (primary PDF unreadable) |
| 2 | Hillaire 2020 atmosphere | Checked | Primary |
| 3 | Jendersie & d'Eon Mie fit | Checked | Primary |
| 4 | Lommel-Seeliger, Hapke, opposition surge | Checked, formulas second-hand | Abstract + secondary |
| 5 | Gravity darkening | Checked with correction | Primary for Altair, Vega; secondary for Regulus, Achernar |
| 6 | Zonal winds, Rhines scale | Checked with correction | Secondary |
| 7 | Ring resonances, edge waves | Checked with corrections | Primary (review papers) |
| 8 | Unlit side of rings | Checked with caveat | Secondary + derived |
| 9 | Enceladus plume, E ring | Partly checked | Abstract; brightness factor not found |
| 10 | Eclipsed Moon | Checked with correction | Abstract |
| 11 | Mars blue sunset | Checked with corrections | Abstract + primary |
| 12 | Methane rainbow | Checked with caveat | Primary (index) + derived (angle) |
| 13 | Comet colour, activity onset | Checked | Primary + secondary |
| 14 | Top-shaped asteroids | Checked with corrections | Primary |
| 15 | Aurora colours | Checked with corrections | Primary + abstract |
| 16 | Lightning rates | Checked with correction | Primary |
| 17 | Nubis and curl-noise citations | Checked | Publisher records, course pages |
| 18 | Bit-reproducible GPU floating point | Checked; evidence gap noted | Primary (specs, papers) |

---

## 1. Cox & Munk 1954: sea-surface slope variance

**Citation.** Cox, C. & Munk, W. (1954). "Measurement of the Roughness of the Sea Surface from Photographs of the Sun's Glitter." *J. Opt. Soc. Am.* 44(11), 838–850. DOI 10.1364/JOSA.44.000838. Companion: "Statistics of the sea surface derived from sun glitter", *J. Marine Research* 13(2), 198–227.

**Key formula** (clean surface, W in m/s):

| Component | Fit | Grade |
|---|---|---|
| Upwind | 0.000 + 3.16 × 10⁻³ W ± 0.004 | Secondary ×2 |
| Crosswind | 0.003 + 1.92 × 10⁻³ W ± 0.002 | Secondary ×2 |
| Total | 0.003 + 5.12 × 10⁻³ W ± 0.004 (r = 0.986) | Secondary |

- Sources: Mobley's Ocean Optics Web Book; Guérin, Capelle & Hartmann 2023, *Remote Sens. Environ.* 288, 113508, DOI 10.1016/j.rse.2023.113508, Eq. 12.
- **W is measured at 41 ft (12.5 m), not the 10 m standard.** Valid range 1–14 m/s (primary abstract).
- Guérin et al. attribute the fits to the JOSA paper.
- Slicks reduce mean-square slope "by a factor of two or three" (primary abstract).

**Does it say what the note claims?** Yes: 0.003 + 0.00512 × wind is right. Add the 12.5 m height and the 1–14 m/s range.

**Later work.** Bréon & Henriot 2006 (*JGR Oceans* 111(C6), C06005, DOI 10.1029/2005JC003343): "near perfect agreement" on mean-square slope from satellite data. Guérin et al. 2023 (150 million IASI observations): compatible, with small departures from linearity at 5–8 m/s. Wentz 1976 (*JGR* 81(9), 1607–1608, DOI 10.1029/JC081i009p01607) criticises the extrapolation of the data.

**Still open.** The primary PDF could not be opened. Slick-surface coefficients are snippet grade only; my test value for the slick total (0.008 + 1.56 × 10⁻³ W) does not equal the sum of the snippet components (1.62), so check the paper before printing any slick formula.

## 2. Hillaire 2020: sky and atmosphere

**Citation.** Hillaire, S. (2020). "A Scalable and Production Ready Sky and Atmosphere Rendering Technique." *Computer Graphics Forum* 39(4), 13–22 (EGSR 2020). DOI 10.1111/cgf.14050.

**Key figures** (Primary, Table 2, PC settings):

| Look-up table | Size |
|---|---|
| Transmittance | 256 × 64 (2D) |
| Multiple scattering | 32 × 32 (2D) |
| Sky-view | 200 × 100 (2D) |
| Aerial perspective | 32³ froxel volume over 32 km |

- Multiple scattering assumes an isotropic phase function for orders ≥ 2, uniform neighbourhood illumination, and sums all orders as F_ms = 1/(1 − f_ms).
- Atmosphere composition can change every frame without heavy table updates.

**Does it say what the note claims?** Yes to both. Small tables: confirmed. Non-Earth: a Mars-like atmosphere and a tiny planet are shown, but the Mars-like parameters are not given and the scope is rocky planets in RGB.

**Limitations stated in the paper.** Hue drift at very high scattering coefficients; error grows with phase anisotropy (RMSE 0.0058 at g = 0, 0.039 at g = 0.8); volumetric shadows need ray marching; no spatially varying atmosphere.

**Related.** Schneegans et al. 2024, "Physically Based Real-Time Rendering of Atmospheres using Mie Theory", *CGF* 43(2), DOI 10.1111/cgf.15010: Henyey-Greenstein is not adequate for Mars; they use tabulated Mie phase functions on the Bruneton model and name Hillaire's model as future work.

## 3. Jendersie & d'Eon 2023: approximate Mie phase function

**Citation.** Jendersie, J. & d'Eon, E. (2023). "An Approximate Mie Scattering Function for Fog and Cloud Rendering." *ACM SIGGRAPH 2023 Talks*, 2 pages. DOI 10.1145/3587421.3595409. Phase function origin: Draine, B. T. (2003), *ApJ* 598(2), 1017–1025, DOI 10.1086/379118.

**Key formulas** (Primary; d = droplet diameter in µm, main range 5 < d < 50):

- g_HG = exp(−0.0990567 / (d − 1.67154))
- g_D = exp(−2.20679 / (d + 3.91029) − 0.428934)
- α = exp(3.62489 − 8.29288 / (d + 5.52825))
- w_D = exp(−0.599085 / (d − 0.641583) − 0.665888)

**Fitted ranges.** Main paper: 5–50 µm. The supplemental adds three more fits: d ≤ 0.1, 0.1–1.5 and 1.5–5 µm (formulas transcribed in the evidence file).

**Assumptions.** Water in air, refractive index 1.333, log-normal diameters (σ = 0.25), averaged over 400–700 nm. The fit does not reproduce fogbow or glory.

**Non-water droplets.** No published extension or re-fit was found for methane, sulphuric acid, tholin haze or Mars dust. Those cases still use tabulated Mie or T-matrix phase functions.

**Still open.** ACM article number (not in the PDF or Crossref).

## 4. Lommel-Seeliger, Hapke, opposition surge

**Citations.** Hapke, B., "Bidirectional reflectance spectroscopy":

| Part | Reference | DOI | Adds |
|---|---|---|---|
| 1 (1981) | *JGR* 86(B4), 3039–3054 | 10.1029/JB086iB04p03039 | Base model |
| 3 (1984) | *Icarus* 59, 41–59 | 10.1016/0019-1035(84)90054-X | Macroscopic roughness |
| 4 (1986) | *Icarus* 67(2), 264–280 | 10.1016/0019-1035(86)90108-9 | Shadow-hiding opposition effect |
| 5 (2002) | *Icarus* 157, 523–534 | 10.1006/icar.2002.6853 | Coherent backscatter |

**Key formulas.** Lommel-Seeliger: I/F = (ϖ₀P/4)·μ₀/(μ₀+μ) (Derived + snippet). Hapke shadow-hiding term: B(g) = B₀ / (1 + (1/h)·tan(g/2)) (Secondary: Fernando et al. 2013, arXiv:1303.4549, attributing it to Hapke 1986/1993).

**Moon.** Buratti, Hillier & Wang 1996, *Icarus* 124(2), 490–499, DOI 10.1006/icar.1996.0225 (Abstract): "The brightness of the Moon increases more than 40% between solar phase angles of 4° and 0°." Surge is 3–4% larger at 0.41 µm than at 1.00 µm and about 10% greater in the highlands.

**Mechanism for the Moon is still debated.** Buratti 1996: mainly shadow hiding. Hapke et al. 2012 (*JGR Planets* 117(E12), DOI 10.1029/2011JE003916): coherent backscatter contributes nearly 40% in the UV, rising to over 60% in the red.

**Saturn's rings.**

| Quantity | Value | Source | Grade |
|---|---|---|---|
| Surge half-width | about 0.1° (BVRI); 0.16° and 0.19° for A and B rings at 338 nm | French et al. 2007, *PASP* 119, 623–642, DOI 10.1086/519982 | Primary |
| Surge amplitude | "increasing by about 1/3 for α < 0.5°", plus a similar rise from 6° to 0.5° | Salo & French 2010, *Icarus* 210, 785–816, DOI 10.1016/j.icarus.2010.07.002, restating French 2007 | Secondary |
| Cassini | Amplitude about 1.3, half-width 0.25° for τ > 0.7 | Déau et al. (Cassini ISS series; see evidence file) | Secondary |

**Still open.** Hapke's B(g) and the Lommel-Seeliger law in a primary text; Poulet 2002 and Franklin & Cook 1965 numbers.

## 5. Gravity darkening

**Citations.** von Zeipel, H. (1924), "The Radiative Equilibrium of a Rotating System of Gaseous Masses", *MNRAS* 84(9), 665–684, DOI 10.1093/mnras/84.9.665. The interferometry papers cite the companion, *MNRAS* 84, 684–701 ("...a slightly oblate rotating star"); cite both. Lucy, L. B. (1967), "Gravity-Darkening for Stars with Convective Envelopes", *Zeitschrift für Astrophysik* 65, 89.

**Exponents.** T_eff ∝ g^β with β = 0.25 (von Zeipel, radiative) and β = 0.08 (Lucy, convective) (Secondary). Lucy's result appears as 0.32 in papers using the flux convention. Che et al. 2011 recommend an empirical β = 0.19. Espinosa Lara & Rieutord 2011 (*A&A* 533, A43, DOI 10.1051/0004-6361/201117252): the exponent depends on rotation and reaches 0.25 only for slow rotators.

**Measured stars.**

| Star | T pole (K) | T equator (K) | R_eq / R_pol (R☉) | β | ω/ω_crit | Source | Grade |
|---|---|---|---|---|---|---|---|
| Altair | 8450 ± 140 | 6860 ± 150 | 2.029 / 1.634 | 0.190 ± 0.012 | 0.923 | Monnier et al. 2007, *Science* 317, 342, DOI 10.1126/science.1143205 | Primary |
| Vega | 10,070 ± 90 | 8,910 ± 130 | 2.726 / 2.418 | 0.231 ± 0.028 | 0.774 | Monnier et al. 2012, *ApJL* 761, L3, DOI 10.1088/2041-8205/761/1/L3 (arXiv:1211.6055) | Primary, re-read by me |
| Vega (older) | 9,988 ± 61 | 7,557 ± 261 | 2.873 / 2.306 | 0.25 fixed | 0.926 | Peterson et al. 2006, *Nature* 440, 896–899, DOI 10.1038/nature04661 | Primary |
| Vega (older) | 10,150 | 7,900 | n/a | 0.25 | 0.91 | Aufdenberg et al. 2006, *ApJ* 645, 664–675, DOI 10.1086/504149 | Secondary |
| Regulus | 15,400 | 10,314 | n/a | 0.25 | n/a | McAlister et al. 2005, *ApJ* 628, 439, DOI 10.1086/430730 | Secondary |
| Regulus | 14,520 | 11,010 | 4.21 / 3.22 | 0.188 | 0.962 | Che et al. 2011, *ApJ* 732, 68, DOI 10.1088/0004-637X/732/2/68 | Secondary |
| Achernar | 17,124 | 12,673 | ratio 1.352 | 0.166 | n/a | Domiciano de Souza et al. 2014, *A&A* 569, A10, DOI 10.1051/0004-6361/201424144 | Snippet (β, ratio secondary) |

**Still open.** Achernar's temperatures against the paper itself (the journal site blocks automated access). The 2003 axis ratio of 1.56 is an apparent value, later revised to about 1.41.

## 6. Zonal winds and the Rhines scale

**Citation.** Rhines, P. B. (1975). "Waves and turbulence on a beta-plane." *J. Fluid Mech.* 69(3), 417–443. DOI 10.1017/S0022112075001504.

**Wind speeds** (Secondary; the original wind papers were not readable):

| Planet | Equatorial jet | Other |
|---|---|---|
| Jupiter | 100–140 m/s within ±15° | Fastest jet at 24°N, 170–180 m/s; 6–7 eastward jets per hemisphere |
| Saturn | 450–470 m/s (Voyager-era rotation period); 370 m/s with the revised period | Extends to about ±30–35°; 17 jets planet-wide |

**Rhines scale.** L = √(U/β) or π·√(2U/β), with β = 2Ω cos φ / a. Both forms are in use; Gastine et al. 2014 (*PEPI* 232, 36, DOI 10.1016/j.pepi.2014.03.011) note the factor of 2 "is not firmly established" and call the agreement with Jupiter and Saturn "relatively good".

**Computed at 30° latitude** (Derived):

| Case | √(U/β) | π·√(U/β) | π·√(2U/β) |
|---|---|---|---|
| Jupiter, U = 40 m/s | 3,030 km | 9,520 km | 13,470 km |
| Jupiter, U = 100 m/s | 4,790 km | 15,050 km | 21,290 km |
| Saturn, U = 100 m/s | 4,520 km | 14,200 km | 20,080 km |

Observed Jupiter mid-latitude jets are about 6° (roughly 7,000 km) wide each. See correction 3.

**Still open.** A direct quote from Vasavada & Showman 2005 on jet widths; the wind numbers in their original papers (Limaye 1986, Porco 2003, Sánchez-Lavega 2000, García-Melendo 2011).

## 7. Ring structure at resonances; edge waves

**Edge-wave wavelength.** λ = 3π·Δa, where Δa is the radial separation between the moon's orbit and the gap edge (the half-width for a centred moon). Primary: Tiscareno, M. S. (2013), "Planetary Rings", in *Planets, Stars and Stellar Systems* vol. 3, DOI 10.1007/978-94-007-5606-9_7, arXiv:1112.3305, with derivation. It credits Cuzzi & Scargle 1985 (*ApJ* 292, 276, DOI 10.1086/163158) and Showalter et al. 1986 (*Icarus* 66, 297–323, DOI 10.1016/0019-1035(86)90160-0). **The note's "3π × gap half-width" is right.**

Worked values (Derived): Daphnis in the 42 km Keeler gap, 3π × 21 ≈ 198 km; Pan in the 325 km Encke gap, 3π × 162.5 ≈ 1,530 km.

**Gaps and moons.**

| Quantity | Value | Grade |
|---|---|---|
| Encke gap | 325 km (half-width 162.5 km) | Secondary (Hahn 2008, citing Porco et al. 2005) |
| Pan | mean radius 14.1 km | NASA page |
| Keeler gap | 42 km | NASA caption |
| Daphnis | mean radius 3.8 km ("8 km across") | NASA page |
| Daphnis vertical edge-wave height | about 1 km | Abstract: Weiss, Porco & Tiscareno 2009, *AJ* 138, 272–286, DOI 10.1088/0004-6256/138/1/272 |

**B-ring outer edge (Mimas 2:1).** Mean radius 117,570 km; m = 2 amplitude varies 4–71 km with a 5.36-year circulation. French et al. 2023, *Icarus* 405, 115678, DOI 10.1016/j.icarus.2023.115678 (Primary).

**A-ring outer edge (Janus 7:6).** Seven-lobed, amplitude 12.8 km, 2006–2010 only. El Moutamid et al. 2016, *Icarus* 279, 125–140, DOI 10.1016/j.icarus.2015.10.025, arXiv:1510.00434 (Abstract).

**Density waves.** Wavenumber k(r) = D_L (r − r_L) / (2πGσ₀r_L), so wavelength shrinks as 1/(r − r_L) and scales with surface density. Tiscareno et al. 2007, *Icarus* 189, 14, DOI 10.1016/j.icarus.2006.12.025, Eq. 5 (Primary). A-ring surface density: 33.7 g/cm² at 124,000 km, rising about 1.3 g/cm² per 1,000 km.

**Still open.** Daphnis radial edge-wave amplitude (not found). The 3π formula in the 1985/1986 originals.

## 8. Rings seen from the unlit side

**Key formula** (Derived, standard Chandrasekhar single scattering): transmitted I/F = (ϖ₀P/4)·μ₀/(μ₀ − μ)·[exp(−τ/μ₀) − exp(−τ/μ)], which peaks at τ_max = [μμ₀/(μ₀ − μ)]·ln(μ₀/μ). For Sun at 20° and viewer at 30° elevation, τ_max = 0.41.

**Citable ring source.** Scharringhausen & Nicholson 2013, *Icarus* 226, 1275–1293, DOI 10.1016/j.icarus.2013.08.010, Appendix A.2: "the transmitted reflectance peaks strongly when µ < τ < µo" (Primary prose; the equation itself was not readable).

**Appearance.** Dollfus & Brunier 1982 (*Icarus* 52(3), 516–525, DOI 10.1016/0019-1035(82)90012-4): on the unlit face "Ring B is 1.8 time less bright than ring A and Cassini division" (Abstract). Verbanac et al. 2005: inner C ring, Cassini Division and F ring "very bright in transmitted light" near ring-plane crossing (Abstract).

**Typical normal optical depths** (Primary: Miller et al. 2024, *Space Sci. Rev.* 220, DOI 10.1007/s11214-024-01104-y): C ring 0.05–0.10 (plateaux 0.5); B ring 0.9 to over 5; Cassini Division 0.10–0.20; A ring about 0.5.

**Does it say what the note claims?** The B ring is dark and brightness peaks at intermediate optical depth: yes. Which ring is brightest depends on geometry; see correction 8. C-ring and Cassini Division particles are also darker, which reduces their advantage.

## 9. Forward scattering: Enceladus plume and E ring

**Grain sizes.**

| Target | Size | Source | Grade |
|---|---|---|---|
| E ring | radius 1.0 ± 0.3 µm, narrow distribution | Showalter, Cuzzi & Larson 1991, *Icarus* 94, 451–473, DOI 10.1016/0019-1035(91)90241-K | Abstract |
| E ring | "distinctly blue" | Nicholson et al. 1996, *Science* 272, 509, DOI 10.1126/science.272.5261.509 | Abstract |
| Plume | median radius 3.1 ± 0.5 µm, from images at phase 173–179° | Ingersoll & Ewald 2011, *Icarus* 216, 492, DOI 10.1016/j.icarus.2011.09.018 | Primary |

Plume mass (1.45 ± 0.5) × 10⁵ kg; E-ring mass (12 ± 5.5) × 10⁸ kg (same paper).

**Brightness against phase angle.** No single factor was found for the E ring or the plume. What is citable: plume mapping uses images above 130° phase; the G ring and D68 show "extremely strong forward-scattering peaks" with nearly constant brightness above 60° scattering angle, fitted by three Henyey-Greenstein terms (Hedman & Stark 2015, *ApJ* 811, 67, DOI 10.1088/0004-637X/811/1/67).

**Jupiter's ring.** Throop et al. 2004, *Icarus* 172, 59–77, DOI 10.1016/j.icarus.2003.12.020.

**Still open.** A measured brightness ratio or phase function for the E ring or plume. If the note gives a specific factor (for example "100 times brighter"), it stays "from memory".

## 10. The eclipsed Moon

**Brightness.**

- Vollmer & Gedzelman 2008, *Applied Optics* 47(34), H52–H61, DOI 10.1364/AO.47.000H52 (Abstract): for a clean molecular atmosphere, irradiance at umbra centre "is reduced by a factor of about 2400"; typical aerosol, ozone, terrain and cloud reduce it by a further factor of order 100.
- Hernitschek, Schmidt & Vollmer 2008, *Applied Optics* 47(34), H62, DOI 10.1364/AO.47.000H62 (Abstract): measured minima of −3.32 and −1.7 mag, about 6 × 10³ and 2.5 × 10⁴ times fainter than full Moon (Derived).
- Strassmeier et al. 2020, *A&A* 635, A156, DOI 10.1051/0004-6361/201936091: 10.75 mag dimming, January 2019.

**Colour.** Red from Rayleigh scattering removing blue along the limb path; ozone Chappuis absorption of orange light gives the blue-turquoise umbral edge (Confirmed: García Muñoz & Pallé 2011, *JQSRT* 112, 1609, DOI 10.1016/j.jqsrt.2011.03.017; Schneegans et al. 2025).

**Volcanic dependence.** Keen 1983, *Science* 222, 1011–1013, DOI 10.1126/science.222.4627.1011: stratospheric aerosols "can profoundly affect the brightness".

**Refraction geometry.** 35′ at the horizon, 70′ for the grazing ray, against a 57′ lunar parallax. Quoted only from a non-peer-reviewed preprint (Mallama, arXiv:2112.08966); the 57′ is Derived.

**Other atmospheres.**

- Tsumura et al. 2014, *ApJ* 789, 122, DOI 10.1088/0004-637X/789/2/122: Galilean moons in Jupiter's shadow at 10⁻⁶–10⁻⁷ of normal brightness in the near-infrared.
- Schneegans et al. 2025, "Physically Based Real-Time Rendering of Eclipses", *CGF*, DOI 10.1111/cgf.70017: predicts eclipses of Phobos and Deimos (unvalidated); on Mars refraction is negligible and aerosol scattering dominates.
- García Muñoz et al. 2012, *ApJ* 755, 103, DOI 10.1088/0004-637X/755/2/103: refraction limits what a transit can probe.
- Nothing found for Titan or Saturn.

## 11. Mars's blue sunsets

**Mechanism.** Ehlers, Chakrabarty & Moosmüller 2014, "Blue moons and Martian sunsets", *Applied Optics* 53(9), 1808, DOI 10.1364/AO.53.001808 (Abstract): the glow "is due to the dominance of near-forward scattering of blue light by dust particles and cannot be explained by a simple, Rayleigh-like selective extinction explanation".

**Dust effective radius.**

| Source | Value | Grade |
|---|---|---|
| Tomasko et al. 1999, *JGR* 104(E4), 8987–9007, DOI 10.1029/1998JE900016 | 1.6 ± 0.15 µm | Snippet + thesis abstract |
| Lemmon et al. 2004, *Science* 306, 1753, DOI 10.1126/science.1104474 | 1.47 ± 0.21 µm (Gusev), 1.52 ± 0.18 µm (Meridiani) | Abstract |
| Lemmon et al. 2015, *Icarus* 251, 96–111, DOI 10.1016/j.icarus.2014.03.029 | about 1 µm (northern summer) to 2 µm (storm onset) | Primary |
| Chen-Chen et al. 2019, *Icarus* 319, 43, DOI 10.1016/j.icarus.2018.09.010 | 0.8–1.5 µm | Primary |

**Single-scattering albedo.** About 0.77 at 436 nm (Wolff et al. 2010, *Icarus* 208, 143, DOI 10.1016/j.icarus.2010.01.010); about 0.975 at 650 nm (0.94 in the Pathfinder analysis).

**Phase function.** Henyey-Greenstein is not adequate (Schneegans et al. 2024). Retrievals use T-matrix cylinders (Wolff et al. 2009, *JGR Planets* 114(E2), DOI 10.1029/2009JE003350); spheres over-estimate backscatter.

**Still open.** Which size and wavelength regime gives a blue solar disc (Ehlers full text not read).

## 12. Rainbow geometry for liquid methane

**Refractive index** (Primary: Martonchik & Orton 1994, "Optical constants of liquid and solid methane", *Applied Optics* 33(36), 8306–8317, DOI 10.1364/AO.33.008306):

| Temperature | n at 0.40 µm | n at 0.58 µm |
|---|---|---|
| 90 K | 1.300 | 1.292 |
| 111 K | 1.280 | 1.273 |

Direct measurements quoted there at about 111 K: 1.286 (0.4 µm), 1.283 (0.6 µm), ± 0.005.

**Bow radius** (Derived, Descartes minimum deviation):

| n | Primary bow | Note |
|---|---|---|
| 1.333 (water) | 42.1° | Secondary bow 50.9° |
| 1.292 | 48.4° | |
| 1.290 | 48.8° | |
| 1.286 | 49.4° | |
| 1.273 | 51.7° | |

**Citable statement.** Zugger et al. 2010, *ApJ* 723(2), 1168–1179, DOI 10.1088/0004-637X/723/2/1168 (Primary, arXiv:1006.3525): rainbow scattering angle 131° for methane droplets, that is a 49° radius, attributed to Bailey 2007 (*Astrobiology* 7, 320, DOI 10.1089/ast.2006.0039). The familiar "at least 49° vs 42.5°" wording is a 2005 NASA outreach piece quoting Les Cowley, not peer-reviewed.

**Still open.** Bailey 2007's own sentence; indices for liquid ethane, nitrogen and mixtures (Badoz et al. 1992, *Rev. Sci. Instrum.* 63, 2967, DOI 10.1063/1.1142594, paywalled).

## 13. Comet coma colour and activity onset

**Green coma, no green tail.** Borsovszky et al. 2021, "Photodissociation of dicarbon: How nature breaks an unusual multiple bond", *PNAS* 118(52), e2113315118, DOI 10.1073/pnas.2113315118 (Primary): C₂ lifetime about 1.6 × 10⁵ s, "∼2 d at 1 AU", which "explains why it is that the coma of a comet is often green, but its tail is not".

**Swan band heads.** 516.5 nm (Δv = 0), 473.7 nm (+1), 563.6 nm (−1). Hema, Pandey & Lambert 2012, *ApJ* 747, 102, DOI 10.1088/0004-637X/747/2/102 (Primary).

**Haser scale lengths for C₂ at 1 au.** Parent 2.2 × 10⁴ km, daughter 6.6 × 10⁴ km, scaling as r². A'Hearn et al. 1995, *Icarus* 118, 223–270, DOI 10.1006/icar.1995.1190, as restated in Hyland et al. 2019, *MNRAS* 484, 1347–1358, DOI 10.1093/mnras/stz075 (Secondary).

**Activity onset.**

| Ice | Distance where sublimation takes over | Source |
|---|---|---|
| Water | about 3 au | Womack, Sarid & Wierzchos 2017, *PASP* 129, 031001, DOI 10.1088/1538-3873/129/973/031001 (Primary) |
| CO₂ | about 12 au | Kelley et al. 2013, *Icarus* 225, 475, DOI 10.1016/j.icarus.2013.04.012 (Primary) |
| CO | about 50 au (vigorous beyond 35 au per Womack) | Same |

**Tails.** Blue ion tail from CO⁺ (Beth et al. 2024, *Comets III* chapter, arXiv:2211.03868). Sodium tail: Cremonese et al. 1997, *ApJ* 490, L199–L202, DOI 10.1086/311040.

**Still open.** Meech & Svoreň 2004 (*Comets II*, pp. 317–336) text not read.

## 14. Top-shaped rubble-pile asteroids

**Spin period at which the ridge forms.**

| Body | Formative or failure spin | Present spin | Density | Source | Grade |
|---|---|---|---|---|---|
| Ryugu | 3.5 h ("twice the current rate") | 7.63 h | 1.19 ± 0.02 g/cm³ | Watanabe et al. 2019, *Science* 364, 268, DOI 10.1126/science.aav8032 | Primary |
| Bennu | 3.6 h (surface "consistent with failure") | 4.296 h | 1190 ± 13 kg/m³ | Scheeres et al. 2019, *Nature Astronomy* 3, 352–361, DOI 10.1038/s41550-019-0721-3 | Primary |

**Idealised limit.** For a cohesionless sphere, P_crit = √(3π/(Gρ)) = 3.30 h / √ρ (ρ in g/cm³): 3.03 h at 1.19, 2.33 h at 2.0, 2.23 h at 2.2 (Derived). The observed 2.2 h spin barrier matches ρ ≈ 2.2 g/cm³. Formula attributed to Pravec & Harris 2000, *Icarus* 148, 12–20, DOI 10.1006/icar.2000.6482 (Secondary).

**Bennu.** Mean diameter 490.06 ± 0.16 m (Barnouin et al. 2019, *Nature Geoscience* 12, 247–252, DOI 10.1038/s41561-019-0330-x; Snippet). Spin acceleration (3.63 ± 0.52) × 10⁻⁶ deg day⁻² (Hergenrother et al. 2019, *Nature Communications* 10, 1291, DOI 10.1038/s41467-019-09213-x; Abstract).

**Formation.**

- Walsh, Richardson & Michel 2008, *Nature* 454, 188–191, DOI 10.1038/nature07078: slow spin-up sheds mass from the equator (the abstract does not use the word "ridge").
- Michel et al. 2020, *Nature Communications* 11, 2655, DOI 10.1038/s41467-020-16433-z: top shapes can form directly by reaccumulation after a disruption.
- Ostro et al. 2006, *Science* 314, 1276–1280, DOI 10.1126/science.1133622: 1999 KW4 primary, spin "approximately 2.8 hours", density about 2 g/cm³, equatorial ridge.

## 15. Aurora emission colours by composition

**Earth.**

| Emission | Wavelength | Radiative lifetime | Peak altitude |
|---|---|---|---|
| Oxygen green | 557.7 nm | 0.7–0.75 s | 100–115 km |
| Oxygen red | 630.0 / 636.4 nm | about 110 s | 150–350 km |
| N₂⁺ first negative | 391.4 / 427.8 nm | about 10⁻⁸ s | 100–120 km |

Sources: Kawamura et al. 2020, *Earth Planets Space* 72, DOI 10.1186/s40623-020-01229-8; Kalogerakis et al. 2009, *Ann. Geophys.* 27, 2183, DOI 10.5194/angeo-27-2183-2009; Whiter et al. 2023, *Ann. Geophys.* 41, 1, DOI 10.5194/angeo-41-1-2023.

**Hydrogen atmospheres.**

- Saturn: Dyudina et al. 2016, "Saturn's aurora observed by the Cassini camera at visible wavelengths", *Icarus* 263, 32–43, DOI 10.1016/j.icarus.2015.05.022 (Primary): "The color of the aurora changes from pink at a few hundred km above the horizon to purple at 1000-1500 km", with "a prominent H-alpha line".
- Jupiter: Ingersoll et al. 1998, *Icarus* 135, 251–264, DOI 10.1006/icar.1998.5971; Vasavada et al. 1999, *JGR* 104(E11), 27133–27142, DOI 10.1029/1999JE001055. Hydrogen lines plus continuum inferred; no colour reported; emission at about 245 km above 1 bar.

**CO₂ atmospheres.**

- Mars, visible: Knutsen et al. 2025, "Detection of visible-wavelength aurora on Mars", *Science Advances* 11(20), eads1563, DOI 10.1126/sciadv.ads1563. Green 557.7 nm line, 18 March 2024 (sol 1094), **101 ± 14 R** from SuperCam and 82 ± 14 R from Mastcam-Z (Primary, re-read by me).
- Mars, ultraviolet: Bertaux et al. 2005, *Nature* 435, 790–794, DOI 10.1038/nature03603. CO Cameron bands and the CO₂⁺ doublet at 289 nm, near 130 km.
- Mars, dayglow (not aurora): Gérard et al. 2020, *Nature Astronomy* 4, 1049–1052, DOI 10.1038/s41550-020-1123-2.
- Venus: Slanger et al. 2001, *Science* 291, 463–465, DOI 10.1126/science.291.5503.463 (green line in the night airglow, no red line); Gray et al. 2014, *Icarus* 233, 342–347, DOI 10.1016/j.icarus.2014.01.029 (strongest after coronal mass ejections).

## 16. Lightning on Jupiter and Saturn

**Jupiter.**

| Quantity | Value | Source | Grade |
|---|---|---|---|
| Global flash rate | 6.1 × 10⁻² km⁻² yr⁻¹, 15 times earlier estimates | Becker et al. 2020, *Nature* 584, 55–58, DOI 10.1038/s41586-020-2532-1 | Primary, re-read by me |
| Earlier optical estimate | about 4 × 10⁻³ km⁻² yr⁻¹ | Little et al. 1999, *Icarus* 142, 306–323, DOI 10.1006/icar.1999.6195; as cited by Becker | Secondary |
| Optical energy per flash | about 10⁵–10⁸ J (Juno); 2 × 10⁸–1.6 × 10¹⁰ J (earlier surveys) | Becker et al. 2020 | Primary |
| Sferics | 377 pole to pole, most above 40°N | Brown et al. 2018, *Nature* 558, 87–90, DOI 10.1038/s41586-018-0156-5 | Abstract |
| Whistlers | over 1,600; peak "up to four lightning strokes per second" | Kolmašová et al. 2018, *Nature Astronomy* 2, 544–548, DOI 10.1038/s41550-018-0442-z | Abstract |

Shallow lightning above the 2-bar level, where liquid water cannot exist, is confirmed (Becker 2020).

**Earth, for comparison.** About 46 flashes per second globally, 2.8 km⁻² yr⁻¹. Planet-wide, Jupiter's rate works out to about 119 flashes per second, roughly 2.6 times Earth's (Derived).

**Saturn.**

| Quantity | Value | Source | Grade |
|---|---|---|---|
| Radio discharge rate, 2010–11 storm | "peak rates larger than ten per second" | Fischer et al. 2011, *Nature* 475, 75–77, DOI 10.1038/nature10205 | Abstract |
| Optical flash rate | 1–2 per minute (southern storms); about 5 per second (2011) | Dyudina et al. 2013, *Icarus* 226, 1020–1037, DOI 10.1016/j.icarus.2013.07.013 | Abstract |
| Optical energy | up to 1.7 × 10⁹ J | Dyudina et al. 2010, *GRL* 37(9), DOI 10.1029/2010GL043188 | Snippet |
| Storm latitude | 35°S planetocentric, and 35°N | Fischer 2011 overview, DOI 10.1553/PRE7s135 | Primary |

## 17. Nubis and curl-noise citations

1. Bridson, R., Hourihan, J. & Nordenstam, M. (2007). "Curl-noise for procedural fluid flow." *ACM Transactions on Graphics* 26(3), Article 46 (Proc. SIGGRAPH 2007). DOI 10.1145/1276377.1276435. **Peer-reviewed.**
2. Schneider, A. (2015). "The Real-time Volumetric Cloudscapes of Horizon: Zero Dawn." In *Advances in Real-Time Rendering in Games, Part I*, ACM SIGGRAPH 2015 Courses. https://advances.realtimerendering.com/s2015/index.html. Course talk. The course page lists Schneider alone; the slide deck adds Nathan Vos.
3. Schneider, A. (2016). "Real-Time Volumetric Cloudscapes." In *GPU Pro 7* (ed. W. Engel), A K Peters/CRC Press. DOI 10.1201/b21261-11. Book chapter. Pages 111–142 in the e-book record; 97–127 in print is snippet grade, so cite the DOI.
4. Schneider, A. (2017). "Nubis: Authoring Real-Time Volumetric Cloudscapes with the Decima Engine." In *Advances in Real-Time Rendering in Games, Part I*, ACM SIGGRAPH 2017 Courses. https://advances.realtimerendering.com/s2017/index.html. Course talk.
5. Schneider, A. (2022). "Nubis, Evolved: Real-Time Volumetric Clouds for Skies, Environments, and VFX." In *Advances in Real-Time Rendering in Games, Part II*, ACM SIGGRAPH 2022 Courses. Course DOI 10.1145/3532720.3546903. Course talk.
6. Schneider, A. (2023). "Nubis³: Methods (and madness) to model and render immersive real-time voxel-based clouds." In *Advances in Real-Time Rendering in Games, Part I*, ACM SIGGRAPH 2023 Courses. Course DOI 10.1145/3587423.3607877 (covers the whole course). Course talk. Often written "Nubis Cubed".

None of the Nubis material is peer-reviewed, and no peer-reviewed paper describing the Nubis method was found. A 2018 Eurographics talk ("Nubis: Realtime Volumetric Cloudscapes in a Nutshell") exists per Schneider's bio; its details were not verified.

## 18. Bit-reproducible floating point across GPUs

**Peer-reviewed work found.**

| Paper | Finding |
|---|---|
| Arteaga, Fuhrer & Hoefler 2014, IEEE IPDPS, 1235–1244, DOI 10.1109/IPDPS.2014.127 | The two sources of non-reproducibility are transcendental functions and non-associative reductions; controlling both gives bit-reproducibility across platforms |
| Zahid, Laguna & Le 2024, SC24-W, 547–557, DOI 10.1109/SCW63240.2024.00077 | 600,000 tests on NVIDIA vs AMD GPUs: discrepancies in 0.98% of double-precision and 9.00% of single-precision runs, largely from math-library calls |
| Collange, Defour, Graillat & Iakymchuk 2015, *Parallel Computing* 49, 83–97, DOI 10.1016/j.parco.2015.09.001 | Bit-exact parallel sums on CPUs and on NVIDIA and AMD GPUs |
| Demmel & Nguyen 2015, *IEEE Trans. Computers* 64(7), 2060–2070, DOI 10.1109/TC.2014.2345391 | Order-independent reproducible summation at about 7n operations |
| Ahrens, Demmel & Nguyen 2020, *ACM TOMS* 46(3), DOI 10.1145/3389360 | Reproducible summation with one pass and one reduction |
| Shanmugavelu et al. 2024, SC24-W, 170–179, DOI 10.1109/SCW63240.2024.00028 | Non-associativity makes GPU kernels non-reproducible run to run |
| Laguna 2020, IEEE IPDPS, 622–633, DOI 10.1109/IPDPS47924.2020.00070 | Random differential testing of floating point across systems |
| Sibidanov, Zimmermann & Glondu 2022, IEEE ARITH, 26–34, DOI 10.1109/ARITH54963.2022.00014 | Correctly rounded math functions (CPU); the only way to make sin/cos/exp identical by construction |

Not peer-reviewed but relevant: Whitehead & Fit-Florea, NVIDIA white paper on IEEE 754 compliance; Gladman, Innocente, Mather & Zimmermann, HAL preprint hal-03141101 (math libraries disagree). Taufer et al. 2010 (IPDPS, DOI 10.1109/IPDPS.2010.5470481) is about accuracy drift, not cross-vendor bit identity.

**What the shader specifications guarantee** (Primary, spec text):

| | WebGPU shading language | Vulkan / SPIR-V |
|---|---|---|
| Add, subtract, multiply | "Correctly rounded", but no rounding mode specified | Correctly rounded; rounding mode can be pinned with an optional feature |
| Divide | 2.5 ULP | 2.5 ULP |
| Square root | Inherited from 1/inverseSqrt (2 ULP) | Same |
| sin, cos | Absolute error up to 2⁻¹¹ | Same |
| atan | 4096 ULP | 4096 ULP |
| Reordering and fusing | Allowed; no opt-out | Allowed unless `NoContraction` is applied |
| Subnormals | May flush to zero | May flush unless pinned with an optional feature |

**Conclusion for the GPU-physics section.**

- Integer or fixed-point arithmetic is the only route that is exact by specification on every tier, including WebGPU.
- On Vulkan a floating-point route exists but is narrow: add, subtract and multiply only, every operation marked `NoContraction`, rounding and subnormals pinned, fixed reduction order, and division, square root, dot products and all built-in functions re-implemented.
- **Evidence gap:** no peer-reviewed study was found demonstrating bit-identical floating-point compute-shader results across GPU vendors under Vulkan or WebGPU.

---

## Still unverified (keep as "from memory")

| Item | What |
|---|---|
| 1 | Slick-surface formulas; anything from the primary Cox & Munk text |
| 3 | ACM article number |
| 4 | Hapke B(g) and Lommel-Seeliger in a primary text; Poulet 2002 and Franklin & Cook 1965 numbers |
| 5 | Achernar pole and equator temperatures in the paper itself |
| 6 | Original wind-profile papers; a Vasavada & Showman quote |
| 7 | Daphnis radial edge-wave amplitude |
| 9 | Any numeric brightness-versus-phase factor for the E ring or plume |
| 10 | Link 1969; refraction geometry in a peer-reviewed source; a 10⁶ dimming figure |
| 11 | Size regime for a blue solar disc |
| 12 | Bailey 2007's own rainbow sentence; ethane and nitrogen indices; the secondary-bow geometry in a source |
| 13 | Meech & Svoreň 2004 text |
| 14 | Bennu diameter in the paper itself; "ridge" wording in Walsh 2008 |
| 16 | Dyudina 2010 flash energy and rate in the paper itself |
| 17 | 2018 Eurographics talk details; GPU Pro 7 print pages |

## Supporting files

All under `/home/user/sse4-citation-check/`.

| File | Contents |
|---|---|
| `agent-notes/items-1-7-13-14-17-18.md` | Quotes, URLs and verdicts for those items |
| `agent-notes/items-2-3-12-15.md` | Same, including all four Jendersie & d'Eon fit sets |
| `agent-notes/items-4-8-9-10-11.md` | Same |
| `agent-notes/items-5-6-16.md` | Same |
| `consensus-threads/A` to `F` | The six Consensus research reports, with their inline citation markers and paper links |

Consensus thread links:

- Items 1–3: https://consensus.app/search/i-am-fact-checking-a-technical-design-note-for-eac/7Y9f3_ZkQfCe53fJMnTH5g/
- Items 4, 8, 9: https://consensus.app/search/i-am-fact-checking-a-technical-design-note-on-rend/Ocqkoy72RzKJxOs-ZsxH8A/
- Items 5, 6, 16: https://consensus.app/search/i-am-fact-checking-a-technical-design-note-on-rend/GeCTCJgzRbO4EULg0J3ndg/
- Items 7, 13, 14: https://consensus.app/search/i-am-fact-checking-a-technical-design-note-on-rend/gHkfYS6tTF-4tFBfFfPBEQ/
- Items 10, 11, 12, 15: https://consensus.app/search/i-am-fact-checking-a-technical-design-note-on-rend/3k8crfq3RFigN5GwO3Jt6g/
- Items 17, 18: https://consensus.app/search/two-bibliographic-literature-tasks-for-a-technical/UPaPgSQRTaGYSOH0yRjjQA/

## Network hosts added to the Default cloud environment

Added during this check, each through an approval prompt; they apply to all future sessions on Default:

arxiv.org, ui.adsabs.harvard.edu, www.nature.com, www.aanda.org, ntrs.nasa.gov, science.nasa.gov, hal.science, www.ebi.ac.uk, api.crossref.org, sebh.github.io, research.nvidia.com, docs.nvidia.com, data-ww3.ifremer.fr, oceanopticsbook.info, www.cs.ubc.ca, advances.realtimerendering.com, www.w3.org, registry.khronos.org, www.lpi.usra.edu.

Four of these did not help automated reading: www.aanda.org and registry.khronos.org answer with a bot check, www.nature.com redirects to a login host that is not allowed, and data-ww3.ifremer.fr and www.lpi.usra.edu refuse the files at the server.
