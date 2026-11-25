# Consensus batch 4: 50 papers (2026-11-25)

Paste everything below the line into a Claude conversation that has the **Consensus** connector, then paste
Claude's whole reply back into the SSE4 citation thread (or save it as `results-04.md` beside this file).

---

You are checking the references of a science-education program. For each numbered paper below, use Consensus
(search by title words, authors and year, or by DOI) to find the paper and read its abstract or text. For each:

1. Does the paper exist as cited? Give the full citation you found: authors, year, title, journal, volume, pages, DOI.
2. What is its key formula, number or finding that bears on the claim?
3. Does it say what we claim? Answer with exactly one status word:
   - `confirmed`: it exists as cited and supports the claim;
   - `wrong citation`: the paper exists but a detail is wrong (year, journal, DOI, author, equation number); say what;
   - `says something else`: the paper does not support the claim, or gives a different number or formula; say what it says;
   - `not found`: Consensus cannot find it (say what you searched).
   If you could only read the abstract and the claim is a detail inside the paper, say so in the note and use
   `confirmed` only when the abstract itself supports it; otherwise use `not found` with the note "abstract only".

Do not guess. Keep each note under 60 words.

Reply with one line per paper, in this exact form, and nothing between the lines:

`key | status | full citation found | key formula or figure | note`

Papers:

**perryman2018**: Perryman, M. 2018, The Exoplanet Handbook, 2nd edn, Cambridge University Press
   DOI as cited: 10.1017/9781108304160
   What we rest on it:
   - const PERRYMAN: &str = "Perryman, M. 2018, The Exoplanet Handbook, 2nd edn, Cambridge University Press: astrometric signature α = (m/M*)(a/d), imaging contrast and inner working angle, timing. doi:10.1017/9781108304160";

**richard2012**: Richard, C. et al. 2012, J. Quant. Spectrosc. Radiat. Transf. 113, 1276 doi:10.1016/j.jqsrt.2011.11.004
   DOI as cited: 10.1016/j.jqsrt.2011.11.004
   What we rest on it:
   - Richard, C. et al. 2012, J. Quant. Spectrosc. Radiat. Transf. 113, 1276
   - Richard, C. et al. 2012, J. Quant. Spectrosc. Radiat. Transf. 113, 1276 doi:10.1016/j.jqsrt.2011.11.004
   - 10.1016/j.jqsrt.2011.11.004

**schwarzschild1975**: Schwarzschild, M. 1975, On the scale of photospheric convection in red giants and supergiants, ApJ 195, 137
   DOI as cited: 10.1086/153313
   What we rest on it:
   - const SCHW75: &str = "Schwarzschild, M. 1975, On the scale of photospheric convection in red giants and supergiants, ApJ 195, 137: cell size follows the pressure scale height, so giants' cells are huge. doi:10.1086/153313";

**skolnik2001**: Skolnik, M. I. 2001, Introduction to Radar Systems, 3rd edn, McGraw-Hill, chapter 2"
   What we rest on it:
   - const R_RADAR: &str = "Radar equation P_r = P G² λ² σ / ((4π)³ R⁴) and effective aperture A_e = G λ² / 4π: Skolnik, M. I. 2001, Introduction to Radar Systems, 3rd edn, McGraw-Hill, chapter 2";

**yashiro2006**: Yashiro, S. et al. 2006, Different power-law indices in the frequency distributions of flares with and without coronal mass ejections, ApJ 650, L143
   DOI as cited: 10.1086/508876
   What we rest on it:
   - const YASHIRO06: &str = "Yashiro, S. et al. 2006, Different power-law indices in the frequency distributions of flares with and without coronal mass ejections, ApJ 650, L143: the share of flares with ejections rises with flare size, to nearly all of the larges…

**bodhaine1999**: Bodhaine, B. A., Wood, N. B., Dutton, E. G. and Slusser, J. R. 1999, On Rayleigh optical depth calculations, J. Atmos
   DOI as cited: 10.1175/1520-0426(1999)016
   What we rest on it:
   - const BOD99: &str = "Bodhaine, B. A., Wood, N. B., Dutton, E. G. and Slusser, J. R. 1999, On Rayleigh optical depth calculations, J. Atmos. Oceanic Technol. 16, 1854: σ = 24π³(n²−1)²/(λ⁴N_s²(n²+2)²)·F; N_s = 2.546899×10¹⁹ cm⁻³ at 288.15 K; dry air 4.5105×10⁻²⁷…
   - /// Bodhaine et al. 1999 (4.5105e-31 m², τ = 0.0971) and Bucholtz 1995.
   - Bodhaine, B. A. et al. 1999, On Rayleigh optical depth calculations, J. Atmos. Oceanic Technol. 16, 1854

**born1999**: Born, M. and Wolf, E. 1999, Principles of Optics, 7th edn doi:10.1017/CBO9781139644181
   DOI as cited: 10.1017/cbo9781139644181
   What we rest on it:
   - Born, M. and Wolf, E. 1999, Principles of Optics, 7th edn, Cambridge University Press, section 8.5.2: a circular aperture's light falls to its first dark ring at 1.22 wavelengths over the aperture, and 83.8% of the light lies inside it.
   - Born, M. and Wolf, E. 1999, Principles of Optics, 7th edn, Cambridge University Press, section 8.5.2: a circular aperture's light falls to its first dark ring at 1.22 wavelengths over the aperture, and 83.8% of the light lies inside it. doi:10.1017/CBO97811396…
   - Born and Wolf 1999, section 8.5.2 (first zero of J1 over pi, 1.2197)

**cioffi1988**: Cioffi, D. F., McKee, C. F. and Bertschinger, E. 1988, Dynamics of radiative supernova remnants, ApJ 334, 252. doi:10.1086/166834"
   DOI as cited: 10.1086/166834
   What we rest on it:
   - const CMB88: &str = "Cioffi, D. F., McKee, C. F. and Bertschinger, E. 1988, Dynamics of radiative supernova remnants, ApJ 334, 252: t_PDS = 1.33×10⁴ E₅₁^(3/14) n₀^(-4/7) yr, R_PDS = 14.0 E₅₁^(2/7) n₀^(-3/7) pc, R = R_PDS (4t/3t_PDS - 1/3)^(3/10). doi:10.1086/1…
   - const CMB88: &str = "Cioffi, D. F., McKee, C. F. and Bertschinger, E. 1988, Dynamics of radiative supernova remnants, ApJ 334, 252. doi:10.1086/166834";
   - - Sedov and Taylor for the blast wave; Cioffi, McKee and Bertschinger (1988) for the later cooling phases; Weaver et al. (1977) for wind-blown bubbles.

**fakhouri2010**: Fakhouri, Ma and Boylan-Kolchin 2010, MNRAS 406, 2267, arXiv:1001.2304
   arXiv as cited: 1001.2304
   What we rest on it:
   - law: "merger rate fitted in our universe, used unchanged on the changed halo counts (Fakhouri, Ma and Boylan-Kolchin 2010, MNRAS 406, 2267, arXiv:1001.2304)",
   - //! the Fakhouri, Ma and Boylan-Kolchin 2010 rate (eq. 1), each a dated
   - /// Fakhouri, Ma and Boylan-Kolchin 2010 eq. 1: mergers per halo per unit z

**duchene2013**: Duchene, G. and Kraus, A. 2013, ARA&A 51, 269, table 1
   arXiv as cited: 1303.3028
   What we rest on it:
   - /// in log mass (Duchene and Kraus 2013).
   - // Duchene and Kraus 2013 table 1 (the same points as the formation suite).
   - block: "Duchene and Kraus 2013, Raghavan 2010".into(),

**held1980**: Held and Hou 1980, J. Atmos. Sci. 37, 515, doi:10.1175/1520-0469(1980)037<0515:NASCIA>2.0.CO;2
   DOI as cited: 10.1175/1520-0469(1980)037
   What we rest on it:
   - //! - Hadley cell edge from Held and Hou (1980): the cell reaches
   - "Hadley cell edge {hadley_raw:.0}° (Held and Hou 1980: g {:.2} m/s², tropopause {:.0} km, spin {:.1} h{}, radius {:.0} km).",
   - | Held and Hou 1980, J. Atmos. Sci. 37, 515, doi:10.1175/1520-0469(1980)037<0515:NASCIA>2.0.CO;2 | Hadley extent, Ro_T | pack |

**komacek2016**: Komacek and Showman 2016, ApJ 821, 16, "Atmospheric circulation of hot Jupiters
   arXiv as cited: 1601.00069
   What we rest on it:
   - "Day-night contrast {dn:.2} of the full contrast (Komacek and Showman 2016): wave time {:.1} h, {:.0} % of a night to cool in.",
   - …Day-night contrast is the ratio of the radiative time to the wave or advection time (Komacek and Showman 2016). Collapse comes from night-side temperature against the frost point of the main gas (Wordsworth 2015). The pattern (substellar convection, Rhines-sc…
   - | Hot Jupiters | super-rotating equatorial jet, hot spot shifted east, day-night contrast rising with temperature | banded giant only | the same timescale ratio (Komacek and Showman 2016); a labelled hot-spot offset |

**lodders2003**: Lodders 2003, doi:10.1086/375492
   DOI as cited: 10.1086/375492
   What we rest on it:
   - pub const L03: &str = "Lodders, K. 2003, ApJ 591, 1220: solar system abundances and condensation temperatures of the elements (host phases, table 8). doi:10.1086/375492";
   - pub const REF_LODDERS: &str = "Lodders, K. 2003, ApJ 591, 1220: condensation temperatures (forsterite and iron about 1350 to 1400 K) and protosolar abundances";
   - /// (protosolar Z = 0.0142 as in Asplund et al. 2009; Lodders 2003 gives Z0 = 0.0149; split by carrier as in Oberg et al.

**mitchell2010**: Mitchell, J. L. and Vallis, G. K. 2010, The transition to superrotation in terrestrial atmospheres, J. Geophys. Res. 115, E12008 https://arxiv.org/abs/1008.1996
   arXiv as cited: 1008.1996
   What we rest on it:
   - //! - **Thermal wind and the thermal Rossby number** (Mitchell and Vallis 2010,
   - "Thermal Rossby number {ro_t:.3} (Mitchell and Vallis 2010): thermal wind {u_t:.0} m/s, used {u:.0} m/s, eddies {eddy:.1} m/s."
   - …a. Hadley extent is sqrt(5/3 · Ro_T). Super-rotation turns on as Ro_T passes about 1 (Mitchell and Vallis 2010). One number, no class. |

**sheth1999**: Sheth and Tormen 1999 mass function, raised or lowered by the cell's overdensity
   What we rest on it:
   - // the mass-weighted mean bias is 1 (Sheth and Tormen 1999).
   - //! - how many haloes of each mass: the Sheth and Tormen 1999 mass function
   - /// Sheth and Tormen 1999 constants (A, a, p).

**smart2005**: Smart and Shea 2005, A review of geomagnetic cutoff rigidities for earth-orbiting spacecraft, Adv. Space Res. 36, 2012. doi:10.1016/j.asr.2004.09.015
   DOI as cited: 10.1016/j.asr.2004.09.015
   What we rest on it:
   - const SMART05: &str = "Smart, D. F. and Shea, M. A. 2005, A review of geomagnetic cutoff rigidities for earth-orbiting spacecraft, Adv. Space Res. 36, 2012: Størmer's cut-off; for vertical arrival R_c = (μ₀ M c / 16π r²) cos⁴λ, about 14.9 GV at Earth's equator…
   - /// equator (Smart and Shea 2005); about 0.39 mSv a year from cosmic rays at
   - …field matters most for thin air, for orbit and for the upper atmosphere's chemistry | Smart and Shea 2005 [M7]; Atri et al. 2013 [M8]; Grießmeier et al. 2005 [M9] |

**wordsworth2015**: Wordsworth, R. 2015, Atmospheric heat redistribution and collapse on tidally locked rocky planets, ApJ 806, 180 https://arxiv.org/abs/1412.5575
   arXiv as cited: 1412.5575
   What we rest on it:
   - /// Wordsworth 2015's grey opacity κ, m²/kg.
   - //! thin radiator [(1−α) F κ p / (4 σ g)]^¼ (Wordsworth 2015, eq. 27), against
   - "Collapse {collapse:.2}: night side {night:.0} K against {gname} freezing at {tc:.0} K (Wordsworth 2015)."

**adams2008**: Adams 2008, "Stars in other universes", doi:10.1088/1475-7516/2008/08/010
   DOI as cited: 10.1088/1475-7516/2008/08/010
   What we rest on it:
   - //! stellar homology (Adams 2008). The lifetime and the age brightening are
   - law: "stellar homology, L ∝ σ G⁴ M³ / k_B⁴ (Adams 2008, doi:10.1088/1475-7516/2008/08/010)",
   - law: "stellar homology, R ∝ G M at a fixed ignition energy (Adams 2008, doi:10.1088/1475-7516/2008/08/010)",

**atri2013**: Atri, Hariharan and Grießmeier 2013, Galactic cosmic ray-induced radiation dose on terrestrial exoplanets, Astrobiology 13, 910. doi:10.1089/ast.2013.1052
   DOI as cited: 10.1089/ast.2013.1052
   What we rest on it:
   - const ATRI13: &str = "Atri, D., Hariharan, B. and Grießmeier, J.-M. 2013, Galactic cosmic ray-induced radiation dose on terrestrial exoplanets, Astrobiology 13, 910: the air column, not the field, is the main shield for a world with thick air. doi:10.1089/ast.…
   - …n air, for orbit and for the upper atmosphere's chemistry | Smart and Shea 2005 [M7]; Atri et al. 2013 [M8]; Grießmeier et al. 2005 [M9] |
   - - [M8] Atri, Hariharan and Grießmeier 2013, Galactic cosmic ray-induced radiation dose on terrestrial exoplanets, Astrobiology 13, 910. doi:10.1089/ast.2013.1052

**feldstein1967**: Feldstein and Starkov 1967, Dynamics of auroral belt and polar geomagnetic disturbances, Planet
   DOI as cited: 10.1016/0032-0633(67)90190-0
   What we rest on it:
   - const FS67: &str = "Feldstein, Y. I. and Starkov, G. V. 1967, Dynamics of auroral belt and polar geomagnetic disturbances, Planet. Space Sci. 15, 209: the auroral oval near 67° to 75° magnetic latitude, moving equatorward as activity rises. doi:10.1016/0032-06…
   - /// Aurorae: Earth's oval near 67° to 75° (Feldstein and Starkov 1967),
   - …on lines. Mars has patchy aurorae over old crustal fields | Newell et al. 2007 [M13]; Feldstein and Starkov 1967 [M14]; Bertaux et al. 2005 [M15] |

**haqqmisra2018**: Haqq-Misra, J. et al. 2018, Demarcating circulation regimes of synchronously rotating terrestrial planets within the habitable zone, ApJ 852, 67 https://arxiv.org/abs/1710.00435
   arXiv as cited: 1710.00435
   What we rest on it:
   - //! - **Regime lengths** (Haqq-Misra et al. 2018, eqs. 8 and 9): the deformation
   - …rrotation) follows from the Rossby deformation and Rhines lengths against the radius (Haqq-Misra et al. 2018). Slow rotators that are not locked then sit on the same curves. |
   - | W7b | Mitchell and Vallis 2010 (Ro_T threshold near 1); Lindzen and Farrell 1980 (the 0.31 factor); Komacek and Showman 2016 (timescale ratio); Haqq-Misra et al. 2018 (regime lengths); Wordsworth 2015 (collapse condition) |

**kivelson2000**: Kivelson, M. G. et al. 2000, Galileo magnetometer measurements
   DOI as cited: 10.1126/science.289.5483.1340
   What we rest on it:
   - const KIV00: &str = "Kivelson, M. G. et al. 2000, Galileo magnetometer measurements: a stronger case for a subsurface ocean at Europa, Science 289, 1340: induced response close to that of a perfect conductor (A about 0.97). doi:10.1126/science.289.5483.1340";
   - /// conductor (Kivelson et al. 2000: A about 0.97); a fresh-water layer barely answers.
   - | Europa, Callisto (induced fields) | settled | A salty ocean in Jupiter's changing field carries currents that make a small field of their own. This is how the oceans were found | Kivelson et al. 2000 [P28]; Zimmer et al. 2000 [P29] |

**kwok1978**: Kwok, S., Purton, C. R. and Fitzgerald, P. M. 1978, On the origin of planetary nebulae, ApJ 219, L125. doi:10.1086/182621
   DOI as cited: 10.1086/182621
   What we rest on it:
   - const KPF78: &str = "Kwok, S., Purton, C. R. and Fitzgerald, P. M. 1978, On the origin of planetary nebulae, ApJ 219, L125: a fast late wind sweeps the slow giant wind into a shell. doi:10.1086/182621";
   - reference("Kwok, S., Purton, C. R. and Fitzgerald, P. M. 1978, On the origin of planetary nebulae, ApJ 219, L125. doi:10.1086/182621"),
   - - Kwok, Purton and Fitzgerald (1978) for the interacting-winds picture of planetary nebulae; Balick (1987) for the shape sequence; Balick and Frank (2002) review of shapes and shaping, including companions.

**mastin2009**: Mastin, L. G. et al. 2009, A multidisciplinary effort to assign realistic source parameters to models of volcanic ash-cloud transport and dispersion during eruptions, J. Volcanol. Geotherm. Res. 186, 10-21
   DOI as cited: 10.1016/j.jvolgeores.2009.01.008
   What we rest on it:
   - //! - Plume height on Earth from Mastin et al. (2009): H = 2.00 V^0.241 km
   - /// Mastin et al. (2009): Earth plume height above the vent, km, for a
   - format!("A {h:.1} km degassing plume from {:.0} kg/s of gas (Mastin et al. 2009 applied to the gas rate, a stand-in), trailing a sulphate haze downwind.", v.outgassing_kg_s)

**siegman1990**: Siegman, A. E. 1990, New developments in laser resonators, Proc. SPIE 1224, 2
   DOI as cited: 10.1117/12.18425
   What we rest on it:
   - Siegman, A. E. 1990, New developments in laser resonators, Proc. SPIE 1224, 2: the beam quality M², the factor by which a real beam spreads faster than a perfect one.
   - Siegman, A. E. 1990, New developments in laser resonators, Proc. SPIE 1224, 2: the beam quality M², the factor by which a real beam spreads faster than a perfect one. doi:10.1117/12.18425 https://doi.org/10.1117/12.18425
   - 10.1117/12.18425

**zimmer2000**: Zimmer, Khurana and Kivelson 2000, Subsurface oceans on Europa and Callisto, Icarus 147, 329. doi:10.1006/icar.2000.6456
   DOI as cited: 10.1006/icar.2000.6456
   What we rest on it:
   - const ZIM00: &str = "Zimmer, C., Khurana, K. K. and Kivelson, M. G. 2000, Subsurface oceans on Europa and Callisto: constraints from Galileo magnetometer observations, Icarus 147, 329: a conducting shell answers a changing field with a dipole of relative stren…
   - | Europa, Callisto (induced fields) | settled | A salty ocean in Jupiter's changing field carries currents that make a small field of their own. This is how the oceans were found | Kivelson et al. 2000 [P28]; Zimmer et al. 2000 [P29] |
   - - [P29] Zimmer, Khurana and Kivelson 2000, Subsurface oceans on Europa and Callisto, Icarus 147, 329. doi:10.1006/icar.2000.6456

**emanuel2003**: Emanuel, K. 2003, Tropical cyclones, Annu. Rev. Earth Planet. Sci. 31, 75 doi:10.1146/annurev.earth.31.100901.141259
   DOI as cited: 10.1146/annurev.earth.31.100901.141259
   What we rest on it:
   - "A tropical cyclone: sea above {:.0} °C, at least {:.0}° from the equator where spin can organise it (Emanuel 2003).",
   - | Emanuel 2003, Annu. Rev. Earth Planet. Sci. 31, 75, doi:10.1146/annurev.earth.31.100901.141259 | potential intensity | pack |
   - Emanuel, K. 2003, Tropical cyclones, Annu. Rev. Earth Planet. Sci. 31, 75

**stromgren1939**: Strömgren, B. 1939, The physical state of interstellar hydrogen, ApJ 89, 526. doi:10.1086/144074"
   DOI as cited: 10.1086/144074
   What we rest on it:
   - const STROMGREN39: &str = "Strömgren, B. 1939, The physical state of interstellar hydrogen, ApJ 89, 526. doi:10.1086/144074";
   - reference("Strömgren, B. 1939, The physical state of interstellar hydrogen, ApJ 89, 526. doi:10.1086/144074"),
   - - Strömgren (1939) on ionised regions round hot stars; Osterbrock and Ferland, *Astrophysics of Gaseous Nebulae and Active Galactic Nuclei* (2nd ed., University Science Books, 2006), the standard text for line emission.

**taylor1950**: Taylor, G. I. 1950, The formation of a blast wave by a very intense explosion, Proc. R. Soc. A 201, 159 (ξ = 1.1517 for γ = 5/3). doi:10.1098/rspa.1950.0049"
   DOI as cited: 10.1098/rspa.1950.0049
   What we rest on it:
   - …"Sedov, L. I. 1959, Similarity and Dimensional Methods in Mechanics, Academic Press; Taylor, G. I. 1950, The formation of a blast wave by a very intense explosion, Proc. R. Soc. A 201, 159 (ξ = 1.1517 for γ = 5/3). doi:10.1098/rspa.1950.0049";
   - reference("Taylor, G. I. 1950, The formation of a blast wave by a very intense explosion, Proc. R. Soc. A 201, 159. doi:10.1098/rspa.1950.0049"),
   - Taylor 1950; Sedov 1959

**tiesinga2021**: Tiesinga, E., Mohr, P. J., Newell, D. B. and Taylor, B. N. 2021, Rev. Mod. Phys. 93, 025010
   DOI as cited: 10.1103/revmodphys.93.025010
   What we rest on it:
   - pub const R_CODATA: &str = "Tiesinga, E., Mohr, P. J., Newell, D. B. and Taylor, B. N. 2021, CODATA recommended values of the fundamental physical constants: 2018, Rev. Mod. Phys. 93, 025010";
   - - Constants: CODATA 2018, Tiesinga et al. 2021, doi:10.1103/RevModPhys.93.025010; IAU 2015 Resolution B3 nominal values, Prsa et al. 2016, doi:10.3847/0004-6256/152/2/41
   - Tiesinga, E., Mohr, P. J., Newell, D. B. and Taylor, B. N. 2021, Rev. Mod. Phys. 93, 025010: CODATA recommended values of the fundamental physical constants, 2018

**balick2002**: Balick, B. and Frank, A. 2002, Shapes and shaping of planetary nebulae, ARA&A 40, 439
   DOI as cited: 10.1146/annurev.astro.40.060401.093849
   What we rest on it:
   - const BF02: &str = "Balick, B. and Frank, A. 2002, Shapes and shaping of planetary nebulae, ARA&A 40, 439: binary companions and fast spin give bipolar and butterfly shapes; concentric rings round many nebulae record earlier pulses of mass loss. doi:10.1146/an…
   - - Kwok, Purton and Fitzgerald (1978) for the interacting-winds picture of planetary nebulae; Balick (1987) for the shape sequence; Balick and Frank (2002) review of shapes and shaping, including companions.

**hartmann1998**: Hartmann, L. et al. 1998, ApJ 495, 385
   What we rest on it:
   - //! (Hartmann et al. 1998, gamma = 1): it spreads, feeds the star and thins,
   - pub const REF_HARTMANN: &str = "Hartmann, L. et al. 1998, ApJ 495, 385: similarity-solution discs, accretion about 1e-8 solar masses a year at 1 Myr";
   - // Hartmann et al. 1998: about 1e-8 a year at 1 Myr (within half a dex).

**mcewen1983**: McEwen, A. S. and Soderblom, L. A. 1983, Two classes of volcanic plumes on Io, Icarus 55, 191 doi:10.1016/0019-1035(83)90075-1
   DOI as cited: 10.1016/0019-1035(83)90075-1
   What we rest on it:
   - format!("A {h:.0} km ballistic plume: gas leaves the vent at {speed:.0} m/s into almost no air, height v²/2g (McEwen and Soderblom 1983 speeds).")
   - Thin or no air: plume height v^2/(2g); vent speeds about 0.5 km/s for long-lived Prometheus-type plumes and about 1.0 km/s for short Pele-type eruptions on Io (McEwen and Soderblom 1983). Heights follow from the formula, not from the paper
   - McEwen, A. S. and Soderblom, L. A. 1983, Two classes of volcanic plumes on Io, Icarus 55, 191

**morton1956**: Morton, B. R., Taylor, G. I. and Turner, J. S. 1956, Turbulent gravitational convection from maintained and instantaneous sources, Proc. R. Soc. A 234, 1-23 doi:10.1098/rspa.1956.0011
   DOI as cited: 10.1098/rspa.1956.0011
   What we rest on it:
   - "A {h:.1} km eruption column from {mer:.2e} kg/s of magma: {:.1} km on Earth (Mastin et al. 2009), × {fac:.2} for this world's gravity, air density and stratification (Morton, Taylor and Turner 1956).",
   - …H proportional to F^(1/4) N^(-3/4) with buoyancy flux F proportional to g MER / rho (Morton, Taylor and Turner 1956); N^2 = g^2/(c_p T) for an isothermal stratosphere at the skin temperature, c_p = 3.5 R/M. The constant is Earth's (Mastin); carrying it to oth…
   - Morton, B. R., Taylor, G. I. and Turner, J. S. 1956, Turbulent gravitational convection from maintained and instantaneous sources, Proc. R. Soc. A 234, 1-23

**carroll2017**: Carroll, B. W. and Ostlie, D. A. 2017, An Introduction to Modern Astrophysics, 2nd edn, Cambridge University Press doi:10.1017/9781108380980
   DOI as cited: 10.1017/9781108380980
   What we rest on it:
   - Carroll, B. W. and Ostlie, D. A. 2017, An Introduction to Modern Astrophysics, 2nd edn, Cambridge University Press
   - Carroll, B. W. and Ostlie, D. A. 2017, An Introduction to Modern Astrophysics, 2nd edn, Cambridge University Press doi:10.1017/9781108380980
   - 10.1017/9781108380980

**glasstone1977**: Glasstone, S. and Dolan, P. J. 1977, The Effects of Nuclear Weapons, 3rd edn doi:10.2172/6852629
   DOI as cited: 10.2172/6852629
   What we rest on it:
   - Glasstone, S. and Dolan, P. J. 1977, The Effects of Nuclear Weapons, 3rd edn
   - Glasstone, S. and Dolan, P. J. 1977, The Effects of Nuclear Weapons, 3rd edn doi:10.2172/6852629
   - 10.2172/6852629

**hayes2016**: Hayes 2016, Annu. Rev. Earth Planet. Sci. 44, 57. doi:10.1146/annurev-earth-060115-012247
   DOI as cited: 10.1146/annurev-earth-060115-012247
   What we rest on it:
   - Hayes 2016, Annu. Rev. Earth Planet. Sci. 44, 57. doi:10.1146/annurev-earth-060115-012247
   - methane and ethane collect near the poles, mostly in the north (Hayes 2016).

**kattenhorn2014**: Kattenhorn and Prockter 2014, Nature Geosci. 7, 762. doi:10.1038/ngeo2245
   DOI as cited: 10.1038/ngeo2245
   What we rest on it:
   - Kattenhorn and Prockter 2014, Nature Geosci. 7, 762. doi:10.1038/ngeo2245

**korenaga2013**: Korenaga 2013, Annu. Rev. Earth Planet. Sci. 41, 117. doi:10.1146/annurev-earth-050212-124208
   DOI as cited: 10.1146/annurev-earth-050212-124208
   What we rest on it:
   - Korenaga 2013, Annu. Rev. Earth Planet. Sci. 41, 117. doi:10.1146/annurev-earth-050212-124208

**leger2004**: Leger et al. 2004, Icarus 169, 499. doi:10.1016/j.icarus.2004.01.001
   DOI as cited: 10.1016/j.icarus.2004.01.001
   What we rest on it:
   - Leger et al. 2004, Icarus 169, 499. doi:10.1016/j.icarus.2004.01.001

**nakamoto1994**: Nakamoto, T. and Nakagawa, Y. 1994, ApJ 421, 640
   What we rest on it:
   - //! (Nakamoto and Nakagawa 1994). Condensation lines (rock, soot, water, CO2,
   - pub const REF_NAKAMOTO: &str = "Nakamoto, T. and Nakagawa, Y. 1994, ApJ 421, 640: midplane temperature of an accreting, irradiated disc";

**solomatov1996**: Solomatov and Moresi 1996, J. Geophys. Res. 101, 4737. doi:10.1029/95JE03361
   DOI as cited: 10.1029/95je03361
   What we rest on it:
   - Solomatov and Moresi 1996, J. Geophys. Res. 101, 4737. doi:10.1029/95JE03361

**turcotte1993**: Turcotte 1993, J. Geophys. Res. 98, 17061. doi:10.1029/93JE01775
   DOI as cited: 10.1029/93je01775
   What we rest on it:
   - Turcotte 1993, J. Geophys. Res. 98, 17061. doi:10.1029/93JE01775

**holman1999**: Holman and Wiegert 1999, AJ 117, 621, doi:10.1086/300695 (planets in binaries)
   DOI as cited: 10.1086/300695
   What we rest on it:
   - //! - **Binaries** (Holman and Wiegert 1999): round one star of a pair, or
   - edit. Circumbinary planets use Holman and Wiegert 1999 limits.
   - - Holman and Wiegert 1999, AJ 117, 621, doi:10.1086/300695 (planets in binaries)

**asplund2009**: Asplund et al. 2009 (already the code's scale)
   What we rest on it:
   - /// C/O by number, from the C+N and alpha budgets (solar 0.55, Asplund et al. 2009).
   - // solar 12 + log O/H = 8.69 and protosolar Z = 0.0142 (Asplund et al. 2009; present-day photosphere 0.0134).
   - //! mass fraction, Asplund et al. 2009), so [X/H] = log10(units / gas):

**turcotte2014**: Turcotte, D. L. and Schubert, G. 2014, Geodynamics, 3rd edition, Cambridge University Press (typical crust 2700, mantle 3300 and sediment 2300 kg/m3 of its worked problems) doi:10.1017/CBO9780511843877
   DOI as cited: 10.1017/cbo9780511843877
   What we rest on it:
   - const TS14: &str = "Turcotte, D. L. and Schubert, G. 2014, Geodynamics, 3rd edn, Cambridge University Press, section 4.5 and table 4.2 (isotope heating rates). doi:10.1017/CBO9780511843877";
   - /// isotope (W/kg) and half-life (Gyr). Heat: Turcotte and Schubert 2014, table 4.2;
   - const NEWTON: &str = "Spherical geometry: heat flow is power over surface area, e.g. Turcotte and Schubert 2014, chapter 4. doi:10.1017/CBO9780511843877";

**ruedas2017**: Ruedas, T. 2017, Geochemistry, Geophysics, Geosystems 18, 3530
   DOI as cited: 10.1002/2017gc006997
   What we rest on it:
   - const R17: &str = "Ruedas, T. 2017, Geochemistry, Geophysics, Geosystems 18, 3530: radioactive heat production of six geologically important nuclides (isotope abundances, half-lives, 26Al heat 0.3583 W/kg). doi:10.1002/2017GC006997";
   - /// shares by mass (from atom abundances) and half-lives: Ruedas 2017, tables 1 and 2.
   - /// 2013) × W per kg of Al-26 (Ruedas 2017, table 2), all from the sheet.

**bertrand2016**: Bertrand and Forget 2016, Nature 540, 86. doi:10.1038/nature19337
   DOI as cited: 10.1038/nature19337
   What we rest on it:
   - //! balance is coldest (Bertrand and Forget 2016's equilibrium; poles or
   - …t low latitude becomes a cold trap and grows by runaway albedo and altitude feedback (Bertrand and Forget 2016; Hamilton et al. 2016) | gap | M1 (equilibrium caps at D1) |
   - …basins because they are lower and the column is thicker. This gives Sputnik Planitia (Bertrand and Forget 2016), Triton's cap, Mars's residual caps and Iapetus's dark side's ice loss without stepping seasons. Surface pressure follows the coldest ice (the vapou…

**bertrand2024**: Bertrand et al. 2024, doi:10.1073/pnas.2408226121 (read there)
   DOI as cited: 10.1073/pnas.2408226121
   What we rest on it:
   - /// Bertrand et al. 2024's test: the same model and world, with only the
   - …l with only the orbit changed gives Pluto (low-latitude ice) and Triton (polar caps) (Bertrand et al. 2024) | READ; the 54° is recalled, standard geometry |
   - - Bertrand et al. 2024, doi:10.1073/pnas.2408226121 (read there);

**kroupa2001**: Kroupa 2001 mass draw on [0.1, 100] solar masses through its exact inverse
   arXiv as cited: astro-ph/0009005
   What we rest on it:
   - /// Kroupa 2001: dN/dM ~ M^-alpha between the edges (solar masses).
   - /// Mean stellar mass of a Kroupa 2001 cluster on 0.1 to 100 solar masses, used
   - "Kroupa, P. 2001, MNRAS 322, 231: initial mass function, slopes 1.3 below 0.5 and 2.3 above. arXiv:astro-ph/0009005";
