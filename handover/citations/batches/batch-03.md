# Consensus batch 3: 50 papers (2026-11-25)

Paste everything below the line into a Claude conversation that has the **Consensus** connector, then paste
Claude's whole reply back into the SSE4 citation thread (or save it as `results-03.md` beside this file).

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

**correa2015**: Correa et al. 2015 (paper I, MNRAS 450, 1514) eqs. 14, 15 and 20 to 23 (constant 1.8837 from eq. B2)
   DOI as cited: 10.1093/mnras/stv689
   What we rest on it:
   - // Milky-Way haloes assemble half their mass near z = 1 to 1.5, clusters near z = 0.5 (Wechsler et al. 2002; Correa et al. 2015 fig. 4).
   - //! with alpha and beta from the cosmology by Correa et al. 2015 (paper I,
   - /// Correa et al. 2015 (paper I, MNRAS 450, 1514) eqs. 14, 15 and 20 to 23 (constant 1.8837 from eq. B2): alpha and beta of the

**cummings2018**: Cummings et al. 2018 (ApJ 866, 21) initial-final mass relation, MIST-based
   What we rest on it:
   - How::Calculated { source: "Cummings et al. 2018, ApJ 866, 21 (white dwarfs); sketch above 8 solar masses".into() },
   - /// Cummings et al. 2018 (ApJ 866, 21) initial-final mass relation, MIST-based
   - // Its end: a white dwarf of about 0.57 solar masses (Cummings et al. 2018).

**johnson2010**: Johnson, J. A. et al. 2010, Giant Planet Occurrence in the Stellar Mass-Metallicity Plane, PASP 122, 905 (f = 0.07 (M/Msun)^1.0 10^(1.2 [Fe/H]), giants within about 2.5 au, so a floor for skimmable giants farther out). doi:10.1086/655775
   DOI as cited: 10.1086/655775
   What we rest on it:
   - // Rises with metal content: Johnson et al. 2010 fit 10^(1.2 [Fe/H]), so
   - // (Johnson et al. 2010: proportional to M; Endl et al. 2006 for M dwarfs).
   - "odds: the ship's sensors where it has sensed a system, else giant-planet odds from the star's mass and metallicity (Johnson et al. 2010, PASP 122, 905; doi:10.1086/655775)",

**kesler2008**: Kesler, S. E. and Wilkinson, B. H. 2008, Geology 36, 255 doi:10.1130/G24317A.1
   DOI as cited: 10.1130/g24317a.1
   What we rest on it:
   - pub const KW08: &str = "Kesler, S. E. and Wilkinson, B. H. 2008, Geology 36, 255: Earth's copper resources estimated from tectonic diffusion of porphyry copper deposits (about 1.7e11 t of copper in porphyry deposits above 3.3 km). doi:10.1130/G24317A.1";
   - /// share is calibrated so that its arc copper is Kesler and Wilkinson 2008's
   - // Kesler and Wilkinson 2008: about 1.7e11 t (1.7e14 kg).

**lindzen1980**: Lindzen, R. S. and Farrell, B. 1980, A simple approximate result for the maximum growth rate of baroclinic instabilities, J. Atmos. Sci. 37, 1648-1654 doi:10.1175/1520-0469(1980)037<1648:ASARFT>2.0.CO;2
   DOI as cited: 10.1175/1520-0469(1980)037
   What we rest on it:
   - //! - **Baroclinic growth** σ = 0.31 f (dU/dz) / N (Lindzen and Farrell 1980)
   - "Baroclinic storms {bc:.2}: growth {:.2} per day (Lindzen and Farrell 1980) against a radiative time of {:.1} days.",
   - | Storm-track cyclones only in some regimes | Smooth | Baroclinic growth rate σ = 0.31 f (∂U/∂z) / N (Lindzen and Farrell 1980) against the radiative damping rate. Weak on Venus, strong on Earth and Mars, no regime list. |

**madau2014**: Madau and Dickinson 2014, equation 15, Salpeter, times 0.63 for a Chabrier IMF
   What we rest on it:
   - /// Redshift of peak cosmic star formation in the Madau and Dickinson 2014 fit
   - "the universe forms stars fastest at z = {zp:.2} (Madau and Dickinson 2014); since then the rate has fallen about tenfold")),
   - // Madau and Dickinson 2014, equation 15, Salpeter, times 0.63 for a Chabrier IMF.

**mittag2023**: Mittag, M., Schmitt, J. H. M. M. and Schröder, K.-P. 2023, Revisiting the cycle-rotation connection for late-type stars https://arxiv.org/abs/2306.05866
   arXiv as cited: 2306.05866
   What we rest on it:
   - const MITTAG23: &str = "Mittag, M., Schmitt, J. H. M. M. and Schröder, K.-P. 2023, Revisiting the cycle-rotation connection for late-type stars: short (inactive) branch log P_cyc = 1.488 + 1.324 log P_rot and long (active) branch log P_cyc = 2.22 + 1.28 log P_…
   - // Cycle: the inactive branch gives the Sun 6.1 years (Mittag et al. 2023 say so too).
   - Mittag, M., Schmitt, J. H. M. M. and Schröder, K.-P. 2023, Revisiting the cycle-rotation connection for late-type stars

**oneill2015**: O'Neill, M. E., Emanuel, K. A. and Flierl, G. R. 2015, Polar vortex formation in giant-planet atmospheres due to moist convection, Nature Geoscience 8, 523 doi:10.1038/ngeo2459
   DOI as cited: 10.1038/ngeo2459
   What we rest on it:
   - //! O'Neill, Emanuel and Flierl 2015).
   - "Rule: internal heat of {:.2} W/m² is below {:.2} W/m², too weak a convective forcing to spin up a polar cyclone, so the {name} pole is a broad, diffuse cap (O'Neill, Emanuel and Flierl 2015).",
   - …cyclones around a central one, as on Jupiter (Brueshaber, Sayanagi and Dowling 2019; O'Neill, Emanuel and Flierl 2015).",

**rieutord2010**: Rieutord, M. and Rincon, F. 2010, The Sun's supergranulation, Living Rev. Solar Phys. 7, 2 doi:10.12942/lrsp-2010-2
   DOI as cited: 10.12942/lrsp-2010-2
   What we rest on it:
   - /// Mm, minutes) and supergranules (Rieutord and Rincon 2010: about 30 Mm,
   - const RR10: &str = "Rieutord, M. and Rincon, F. 2010, The Sun's supergranulation, Living Rev. Solar Phys. 7, 2: cells about 30 Mm across, lifetimes one to two days, horizontal flows about 300 to 400 m/s, outlined by the magnetic network. doi:10.12942/lrsp-2010…
   - Rieutord, M. and Rincon, F. 2010, The Sun's supergranulation, Living Rev. Solar Phys. 7, 2

**sanchezlavega2017**: Sánchez-Lavega, A. et al. 2017, The atmospheric dynamics of Venus, Space Sci. Rev. 212, 1541-1616 doi:10.1007/s11214-017-0389-x
   DOI as cited: 10.1007/s11214-017-0389-x
   What we rest on it:
   - …super-rotation strength from Ro_T (Mitchell and Vallis 2010; Gierasch-type mechanism, Sánchez-Lavega et al. 2017); signed spin; deck from production and removal; acid virga |
   - | Sánchez-Lavega et al. 2017, Space Sci. Rev. 212, 1541, doi:10.1007/s11214-017-0389-x | Venus dynamics | pack |
   - …oi:10.1063/1.1541418); drift wind 60 m/s is Venus's cloud-level super-rotation order (Sánchez-Lavega et al. 2017, Space Science Reviews 212, 1541. doi:10.1007/s11214-017-0389-x), a default until a weather stage gives winds

**sutton1971**: Sutton, K. and Graves, R. A. 1971, A general stagnation-point convective heating equation for arbitrary gas mixtures, NASA TR R-376. https://ntrs.nasa.gov/citations/19720003329
   What we rest on it:
   - "aerocapture: one ballistic grazing pass (Vinh, Busemann and Culp 1980), heating by Sutton and Graves 1971",
   - //! RK4 at a fixed step (deterministic). Heating: Sutton and Graves 1971,
   - //! - Heating at the stagnation point: Sutton and Graves 1971 (NASA TR R-376),

**tout1996**: Tout et al. 1996 quote their Z = 0.02, 1 solar-mass fit as L = 0.698, R = 0.888
   What we rest on it:
   - How::Calculated { source: "Hurley, Pols and Tout 2000, MNRAS 315, 543; Tout et al. 1996, MNRAS 281, 257".into() },
   - //! with Tout et al. 1996 (MNRAS 281, 257) for the ZAMS. Transcribed mechanically from
   - //! 543), main-sequence part, with the zero-age fits of Tout et al. 1996 (MNRAS

**vasilyev2024**: Vasilyev, V. et al. 2024, Sun-like stars produce superflares roughly once per century, Science https://arxiv.org/abs/2412.12265
   arXiv as cited: 2412.12265
   What we rest on it:
   - /// k = N(>E_REF) E_REF / L_X, fixed by Vasilyev et al. 2024 (8.63e-3 per
   - const VAS24: &str = "Vasilyev, V. et al. 2024, Sun-like stars produce superflares roughly once per century, Science: (8.63 ± 0.20)×10⁻³ per year above 10³⁴ erg for 5000 to 6500 K stars with rotation over 20 days; energy spread dN/dE ∝ E^(−1.97 ± 0.30), consist…
   - param("alpha", D::NONE, 2.0, (1.1, 3.0), "index of the energy spread dN/dE ∝ E^−α (Shibayama 2013: 2.0; Vasilyev 2024: 1.97 ± 0.30)"),

**adams1997**: Adams and Laughlin 1997, Rev. Mod. Phys. 69, 337
   What we rest on it:
   - //! far-future eras follow Adams and Laughlin 1997 and are labelled speculative.
   - /// Star formation ends, log10 years (Adams and Laughlin 1997: about 1e14).
   - "gas is used up and star formation ends; the longest-lived red dwarfs shine on (Adams and Laughlin 1997)".into()),

**andrews2013**: Andrews et al. 2013, ApJ 771, 129 (disc mass scales with stellar mass)"
   What we rest on it:
   - // Andrews and Martini 2013 (direct method): 12 + log O/H = 8.798 - log(1 + (10^8.901 / M*)^0.64);
   - pub const REF_ANDREWS: &str = "Andrews, S. M. et al. 2010, ApJ 723, 1241 (characteristic radii 14 to 200 AU); Andrews et al. 2013, ApJ 771, 129 (disc mass scales with stellar mass)";
   - …Lake and Negroponte 1982. Tuned against Behroozi et al. 2019, Renzini and Peng 2015, Andrews and Martini 2013 and Madau and Dickinson 2014 (package log G3)

**blanton2007**: Blanton, M. R. and Roweis, S. 2007, K-corrections and filter transformations in the ultraviolet, optical, and near infrared, AJ 133, 734
   DOI as cited: 10.1086/510127
   What we rest on it:
   - pub const R_BR07: &str = "Blanton, M. R. and Roweis, S. 2007, K-corrections and filter transformations in the ultraviolet, optical, and near infrared, AJ 133, 734: effective wavelengths and AB minus Vega offsets (Table 1). doi:10.1086/510127";
   - /// (Blanton and Roweis 2007 Table 1, Vega system; Ramírez et al. 2012 for
   - Blanton, M. R. and Roweis, S. 2007, K-corrections and filter transformations in the ultraviolet, optical, and near infrared, AJ 133, 734: Table 1, effective wavelengths and AB minus Vega offsets

**brace1991**: Brace, L. H. and Kliore, A. J. 1991, The structure of the Venus ionosphere, Space Sci. Rev. 55, 81
   DOI as cited: 10.1007/bf00177136
   What we rest on it:
   - const BRACE91: &str = "Brace, L. H. and Kliore, A. J. 1991, The structure of the Venus ionosphere, Space Sci. Rev. 55, 81: the ionopause sits where the ionosphere's pressure balances the solar wind's, a few hundred km up on Venus. doi:10.1007/BF00177136";
   - /// (Venus ionopause 250 to 400 km, Brace and Kliore 1991; Mars 300 to 800 km);
   - Brace, L. H. and Kliore, A. J. 1991, Space Sci. Rev. 55, 81

**mardling2001**: Mardling, R. A. and Aarseth, S. J. 2001, MNRAS 321, 398 (Tidal interactions in star cluster simulations)
   DOI as cited: 10.1046/j.1365-8711.2001.03974.x
   What we rest on it:
   - "{} the pair {inner}: {r:.1} times its orbit; it needs more than {c:.1} (Mardling and Aarseth 2001){}",
   - "Mardling, R. A. and Aarseth, S. J. 2001, MNRAS 321, 398 (Tidal interactions in star cluster simulations): stability criterion for hierarchical triples, used here as an order of magnitude";
   - // (Mardling and Aarseth 2001, order of magnitude).

**shibayama2013**: Shibayama, T. et al. 2013, Superflares on solar-type stars observed with Kepler I, ApJS 209, 5
   arXiv as cited: 1308.1480
   What we rest on it:
   - const SHIB13_T: &str = "Shibayama, T. et al. 2013, Superflares on solar-type stars observed with Kepler I, ApJS 209, 5: flare light as a 10 000 K black body. https://arxiv.org/abs/1308.1480";
   - const SHIB13: &str = "Shibayama, T. et al. 2013, Superflares on solar-type stars observed with Kepler I, ApJS 209, 5: dN/dE ∝ E^(−2.0) for slow rotators, −2.2 for all G dwarfs; fast rotators flare far more often. https://arxiv.org/abs/1308.1480";
   - param("alpha", D::NONE, 2.0, (1.1, 3.0), "index of the energy spread dN/dE ∝ E^−α (Shibayama 2013: 2.0; Vasilyev 2024: 1.97 ± 0.30)"),

**vinh1980**: Vinh, N. X., Busemann, A. and Culp, R. D. 1980, Hypersonic and Planetary Entry Flight Mechanics, University of Michigan Press
   What we rest on it:
   - "aerocapture: one ballistic grazing pass (Vinh, Busemann and Culp 1980), heating by Sutton and Graves 1971",
   - //! grazing column; Vinh, Busemann and Culp 1980, Hypersonic and Planetary
   - …ntial atmosphere: column rho_p sqrt(2 pi r_p H) and ln(v_in/v_out) = column/(2 beta) (Vinh, N. X., Busemann, A. and Culp, R. D. 1980, Hypersonic and Planetary Entry Flight Mechanics, University of Michigan Press); heat load q_peak sqrt(4 pi r_p H)/v follows fr…

**wyman2013**: Wyman, C., Sloan, P.-P. and Shirley, P. 2013, Journal of Computer Graphics Techniques 2(2), 1 https://jcgt.org/published/0002/02/01/
   What we rest on it:
   - /// (Wyman, Sloan and Shirley 2013), summed over `from_nm..=to_nm` in
   - // Wyman, Sloan and Shirley (2013), "Simple Analytic Approximations to the CIE XYZ
   - Wyman, C., Sloan, P.-P. and Shirley, P. 2013, Journal of Computer Graphics Techniques 2(2), 1

**clem2000**: Clem, J. M. and Dorman, L. I. 2000, Neutron monitor response functions, Space Sci. Rev. 93, 335
   DOI as cited: 10.1023/a:1026508915269
   What we rest on it:
   - const CLEM00: &str = "Clem, J. M. and Dorman, L. I. 2000, Neutron monitor response functions, Space Sci. Rev. 93, 335: ground count rate against cut-off follows N(R_c) = N₀ (1 − exp(−α R_c^−k)); attenuation in air about 140 g/cm² (barometric coefficient about…
   - Clem, J. M. and Dorman, L. I. 2000, Space Sci. Rev. 93, 335
   - Clem, J. M. and Dorman, L. I. 2000, Space Sci. Rev. 93, 335 doi:10.1023/A:1026508915269

**eady1949**: Eady, E. T. 1949, Long waves and cyclone waves, Tellus 1(3), 33-52 doi:10.1111/j.2153-3490.1949.tb01265.x
   DOI as cited: 10.1111/j.2153-3490.1949.tb01265.x
   What we rest on it:
   - /// Most unstable wavelength in deformation radii (Eady 1949).
   - "A storm-track cyclone near {st:.0}°: baroclinic waves about {:.0} km long (Eady 1949) travel east with the westerlies and live days.",
   - Storm-track cyclones as baroclinic waves (Eady 1949; Chang, Lee and Swanson 2002): wavelength 3.9 N H / f; life, wind and winter boost are Earth-like stand-ins (docs/fudges.md)

**freytag2002**: Freytag, B., Steffen, M. and Dorch, B. (2002). Spots on the surface of Betelgeuse
   DOI as cited: 10.1002/1521-3994(200208)323:3/4
   What we rest on it:
   - const FREY02: &str = "Freytag, B., Steffen, M. and Dorch, B. 2002, Spots on the surface of Betelgeuse: results from new 3D stellar convection models, Astron. Nachr. 323, 213: red supergiants have only a few giant convection cells across their faces. doi:10.100…
   - { text: "Freytag, B., Steffen, M. and Dorch, B. (2002). Spots on the surface of Betelgeuse: results from new 3D stellar convection models. Astron. Nachr. 323, 213 (a red supergiant's few giant cells raise and lower its visible edge).", doi: "10.1002/1521-3994(…

**hayashi1981**: Hayashi 1981, a port
   What we rest on it:
   - "places: the system's bodies and belts; fuel by the pack's rules (a giant to skim, ice beyond the snow line at 2.7 au times the square root of the star's luminosity, Hayashi 1981, a port)",
   - /// The snow line at one solar luminosity, au; it scales as √L (Hayashi 1981).

**lada2003**: Lada and Lada 2003
   What we rest on it:
   - …ne density 0.1 solar masses per pc^3 (McKee et al. 2015); cluster mass function M^-2 (Lada and Lada 2003)
   - embedded cluster masses dN/dM proportional to M^-2 from 10 to 1e5 solar masses (Lada and Lada 2003), so per star proportional to M^-1

**liou2002**: Liou, K. N. 2002, An Introduction to Atmospheric Radiation, 2nd ed. doi:10.1016/S0074-6142(02)80015-4
   DOI as cited: 10.1016/s0074-6142(02)80015-4
   What we rest on it:
   - Liou, K. N. 2002, An Introduction to Atmospheric Radiation, 2nd ed.
   - Liou, K. N. 2002, An Introduction to Atmospheric Radiation, 2nd ed. doi:10.1016/S0074-6142(02)80015-4
   - 10.1016/S0074-6142(02)80015-4

**nichols2011**: Nichols, J. D. 2011, Magnetosphere-ionosphere coupling in Jupiter's middle magnetosphere, J. Geophys. Res. 116 (arXiv:1108.3219), eq. 24
   arXiv as cited: 1108.3219
   What we rest on it:
   - const NICH11: &str = "Nichols, J. D. 2011, Magnetosphere-ionosphere coupling in Jupiter's middle magnetosphere, J. Geophys. Res. 116 (arXiv:1108.3219), eq. 24: ρ_H / R = (2π Σ_P* B² R² / Ṁ)^¼ with Σ_P* the effective Pedersen conductance of one hemisphere; abou…
   - // Jupiter, Σ* 0.14 S, Ṁ 1000 kg/s: about 30 R_J (Nichols 2011, after Hill 2001).

**oke1983**: Oke, J. B. and Gunn, J. E. 1983, Secondary standard stars for absolute spectrophotometry, ApJ 266, 713
   DOI as cited: 10.1086/160817
   What we rest on it:
   - /// in cgs (Oke and Gunn 1983, a definition).
   - pub const R_OKE83: &str = "Oke, J. B. and Gunn, J. E. 1983, Secondary standard stars for absolute spectrophotometry, ApJ 266, 713: the AB magnitude m = −2.5 log₁₀ f_ν − 48.60 (cgs). doi:10.1086/160817";
   - Oke, J. B. and Gunn, J. E. 1983, Secondary standard stars for absolute spectrophotometry, ApJ 266, 713: the AB magnitude, m = −2.5 log10 f_ν − 48.60 (cgs), zero point 3631 Jy

**trampedach2013**: Trampedach, R. et al. 2013, ApJ 769, 18 doi:10.1088/0004-637X/769/1/18
   DOI as cited: 10.1088/0004-637x/769/1/18
   What we rest on it:
   - const TRAMP13: &str = "Trampedach, R., Asplund, M., Collet, R., Nordlund, Å. and Stein, R. F. 2013, A grid of three-dimensional stellar atmosphere models of solar metallicity. I. General properties, granulation, and atmospheric expansion, ApJ 769, 18: granule…
   - Trampedach, R. et al. 2013, ApJ 769, 18
   - Trampedach, R. et al. 2013, ApJ 769, 18 doi:10.1088/0004-637X/769/1/18

**zarka2001**: Zarka, P., Treumann, R. A., Ryabov, B. P. and Ryabov, V. B. 2001, Magnetically-driven planetary radio emissions and application to extrasolar planets, Astrophys
   DOI as cited: 10.1023/a:1012221527425
   What we rest on it:
   - const ZARKA01: &str = "Zarka, P., Treumann, R. A., Ryabov, B. P. and Ryabov, V. B. 2001, Magnetically-driven planetary radio emissions and application to extrasolar planets, Astrophys. Space Sci. 277, 293. doi:10.1023/A:1012221527425";
   - "Zarka, P. et al. 2001, Magnetically-driven planetary radio emissions and application to extrasolar planets, Astrophys. Space Sci. 277, 293. doi:10.1023/A:1012221527425",

**andrews2010**: Andrews, S. M. et al. 2010, ApJ 723, 1241 (characteristic radii 14 to 200 AU)
   What we rest on it:
   - pub const REF_ANDREWS: &str = "Andrews, S. M. et al. 2010, ApJ 723, 1241 (characteristic radii 14 to 200 AU); Andrews et al. 2013, ApJ 771, 129 (disc mass scales with stellar mass)";

**calvari1998**: Calvari, S. and Pinkerton, H. 1998, JGR 103, 27291
   DOI as cited: 10.1029/97jb03388
   What we rest on it:
   - //! (Greeley 1971; Calvari and Pinkerton 1998: Etna and Hawaii tubes on
   - "Calvari, S. and Pinkerton, H. 1998, JGR 103, 27291: formation of lava tubes and extensive flow fields during the 1991-1993 eruption of Mount Etna. doi:10.1029/97JB03388",
   - …es a forming strength of {forming:.2} (tubes form best near {best:.0}°, Greeley 1971; Calvari and Pinkerton 1998). Gravity {g:.2} m/s² lets a roof span about {w:.0} m ({ratio:.1} times Earth's; Blair et al. 2017). {kept_pc:.0}% of the roof still stands: {imp}{…

**casagrande2011**: Casagrande et al. 2011
   What we rest on it:
   - // -0.06 after recalibration (Casagrande et al. 2011), spread about

**draine2003**: Draine, B. T. 2003, Interstellar dust grains, ARA&A 41, 241. doi:10.1146/annurev.astro.41.011802.094840
   DOI as cited: 10.1146/annurev.astro.41.011802.094840
   What we rest on it:
   - …tion falls with wavelength: A_λ/A_V ≈ (λ/0.55 µm)^−1.75 in the near and mid infrared (Draine, B. T. 2003, Interstellar dust grains, ARA&A 41, 241. doi:10.1146/annurev.astro.41.011802.094840); the optical shape follows Cardelli, Clayton and Mathis 1989, ApJ 345…

**fatuzzo2008**: Fatuzzo and Adams 2008
   What we rest on it:
   - birth-cluster far-ultraviolet field, median about 1000 G0 (Fatuzzo and Adams 2008)

**gray1989**: Gray, D. F. and Nagel, T. 1989, The granulation boundary in the H-R diagram, ApJ 341, 421
   DOI as cited: 10.1086/167505
   What we rest on it:
   - const GN89: &str = "Gray, D. F. and Nagel, T. 1989, The granulation boundary in the H-R diagram, ApJ 341, 421: no granulation in stars hotter than about early F. doi:10.1086/167505";

**gray2005**: Gray, D. F. 2005, The Observation and Analysis of Stellar Photospheres, 3rd ed. doi:10.1017/CBO9780511614576
   DOI as cited: 10.1017/cbo9780511614576
   What we rest on it:
   - Gray, D. F. 2005, The Observation and Analysis of Stellar Photospheres, 3rd ed.
   - Gray, D. F. 2005, The Observation and Analysis of Stellar Photospheres, 3rd ed. doi:10.1017/CBO9780511614576
   - 10.1017/CBO9780511614576

**gray2009**: Gray, R. O. and Corbally, C. J. 2009, Stellar Spectral Classification, Princeton University Press
   What we rest on it:
   - Gray, R. O. and Corbally, C. J. 2009, Stellar Spectral Classification, Princeton University Press: which absorption lines mark each spectral class (Balmer lines strongest near A0; Ca II H and K, the G band and metal lines in G and K; TiO bands in M; He I in B,…

**greeley1971**: Greeley, R. 1971, Modern Geology 2, 207
   What we rest on it:
   - //! (Greeley 1971; Calvari and Pinkerton 1998: Etna and Hawaii tubes on
   - "Greeley, R. 1971, Modern Geology 2, 207: lava tubes and channels in the lunar Marius Hills and on Hawaii; tubes form in fluid pahoehoe on gentle slopes.",
   - …slope:.1}° gives a forming strength of {forming:.2} (tubes form best near {best:.0}°, Greeley 1971; Calvari and Pinkerton 1998). Gravity {g:.2} m/s² lets a roof span about {w:.0} m ({ratio:.1} times Earth's; Blair et al. 2017). {kept_pc:.0}% of the roof still…

**hill1979**: Hill, T. W. 1979, Inertial limit on corotation, J. Geophys. Res. 84, 6554
   DOI as cited: 10.1029/ja084ia11p06554
   What we rest on it:
   - const HILL79: &str = "Hill, T. W. 1979, Inertial limit on corotation, J. Geophys. Res. 84, 6554: plasma loaded at Ṁ stops corotating beyond ρ_H, where the ionosphere's Pedersen current can no longer supply the torque. doi:10.1029/JA084iA11p06554";

**howell2006**: Howell, S. B. 2006, Handbook of CCD Astronomy, 2nd edn, Cambridge University Press, chapter 4 (the CCD equation). doi:10.1017/CBO9780511807909"
   DOI as cited: 10.1017/cbo9780511807909
   What we rest on it:
   - const R_PHOTON: &str = "Photon-counting signal to noise, S t / √((S + B + D) t): Howell, S. B. 2006, Handbook of CCD Astronomy, 2nd edn, Cambridge University Press, chapter 4 (the CCD equation). doi:10.1017/CBO9780511807909";

**jenkins2010**: Jenkins, J. M. et al. 2010, Overview of the Kepler science processing pipeline, ApJ 713, L87
   DOI as cited: 10.1088/2041-8205/713/2/l87
   What we rest on it:
   - const JENKINS10: &str = "Jenkins, J. M. et al. 2010, Overview of the Kepler science processing pipeline, ApJ 713, L87: the 7.1σ detection threshold. doi:10.1088/2041-8205/713/2/L87";

**karkoschka2010**: Karkoschka, E. and Tomasko, M. G. 2010, Icarus 205, 674 doi:10.1016/j.icarus.2009.07.044
   DOI as cited: 10.1016/j.icarus.2009.07.044
   What we rest on it:
   - Karkoschka, E. and Tomasko, M. G. 2010, Icarus 205, 674
   - Karkoschka, E. and Tomasko, M. G. 2010, Icarus 205, 674 doi:10.1016/j.icarus.2009.07.044
   - 10.1016/j.icarus.2009.07.044

**landis2003**: Landis 2003, AIP Conf. Proc. 654, 1193. doi:10.1063/1.1541418
   DOI as cited: 10.1063/1.1541418
   What we rest on it:
   - …level pressure and 0 to 50 °C, where a breathable-gas envelope floats in heavier air (Landis 2003, AIP Conf. Proc. 654, 1193. doi:10.1063/1.1541418); drift wind 60 m/s is Venus's cloud-level super-rotation order (Sánchez-Lavega et al. 2017, Space Science Revie…

**lichtenberg2016**: Lichtenberg et al. 2016
   What we rest on it:
   - rough: most systems are born with less Al-26 than the Sun had (Gounelle 2015; Lichtenberg et al. 2016). Labelled rough

**lightfoot2017**: Lightfoot 2017, Nickel Sulfide Ores and Impact Melts, Elsevier. doi:10.1016/C2014-0-02315-6
   DOI as cited: 10.1016/c2014-0-02315-6
   What we rest on it:
   - the Sudbury nickel ores settled out of an impact melt sheet (Lightfoot 2017, Nickel Sulfide Ores and Impact Melts, Elsevier. doi:10.1016/C2014-0-02315-6)

**luhmann1991**: Luhmann, J. G. and Cravens, T. E. 1991, Magnetic fields in the ionosphere of Venus, Space Sci. Rev. 55, 201
   DOI as cited: 10.1007/bf00177137
   What we rest on it:
   - const LUH95: &str = "Luhmann, J. G. and Cravens, T. E. 1991, Magnetic fields in the ionosphere of Venus, Space Sci. Rev. 55, 201: induced magnetospheres of unmagnetised bodies with air. doi:10.1007/BF00177137";

**moore2004**: Moore, J. M. et al. 2004, in Jupiter
   What we rest on it:
   - Moore, J. M. et al. 2004, in Jupiter: The Planet, Satellites and Magnetosphere, Cambridge, 397: Callisto, a saturated, tectonically dead ice surface. https://ui.adsabs.harvard.edu/abs/2004jpsm.book..397M

**owen2012**: Owen, J. E., Clarke, C. J. and Ercolano, B. 2012, MNRAS 422, 1880
   What we rest on it:
   - pub const REF_OWEN: &str = "Owen, J. E., Clarke, C. J. and Ercolano, B. 2012, MNRAS 422, 1880: X-ray photoevaporation, about 1e-9 to 1e-8 solar masses a year";
   - // Photoevaporation: X-ray about 1e-9 for a solar-mass star (Owen 2012), plus
