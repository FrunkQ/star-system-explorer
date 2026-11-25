# Citations: aurora drawing (track VP-AU, visual polish design section 16)

References in `packages/renderer/src/aurora.ts` and the `aurora` pass (`passes.ts`), written 24 Nov 2026 for
VP-AU5 part 1. None has been fetched yet; each says what the code relies on. The heights, brightnesses and widths
are the oxygen stand-in's (docs/fudges.md, "Aurora stand-in") and go when MF9 part 2 gives the kernel's own.

| # | Reference | Relied on | Status | Checked |
|---|---|---|---|---|
| AU1 | Hunten, Roach and Chamberlain 1956, A photometric unit for the airglow and aurora, J. Atmos. Terr. Phys. 8, 345. doi:10.1016/0021-9169(56)90111-8 | the rayleigh: 10¹⁰ photons s⁻¹ m⁻² of column (10⁶ per cm²), into 4π | unverified | the unit is textbook; the DOI not fetched |
| AU2 | Chamberlain 1961, Physics of the Aurora and Airglow, Academic Press (AGU reprint doi:10.1029/SP041) | green line peak near 110 km, red line near 250 km; red a fraction of green in a normal oval | unverified | book not online |
| AU3 | Rees 1989, Physics and Chemistry of the Upper Atmosphere, Cambridge. doi:10.1017/CBO9780511573118 | typical oval brightness about 10 kR in 557.7 nm | unverified | book not online |
| AU4 | Feldstein and Starkov 1967, Dynamics of auroral belt and polar geomagnetic disturbances, Planet. Space Sci. 15, 209. doi:10.1016/0032-0633(67)90190-0 | the oval's width of a few degrees (stand-in 5°) | unverified | DOI not fetched |
| AU5 | Wyman, Sloan and Shirley 2013, Simple analytic approximations to the CIE XYZ color matching functions, JCGT 2(2), 1. https://jcgt.org/published/0002/02/01/ | the multi-lobe fit's coefficients (`cieXyz`) | unverified | coefficients written from memory; the test checks only that 557.7 nm is green, 630.0 nm red and 427.8 nm blue |
| AU6 | CIE 191:2010, Recommended system for mesopic photometry based on visual performance | colour vision fading between about 0.001 and 0.01 cd/m² (`COLOUR_VISION_CD_M2`) | unverified | standard not online; the range is the usual statement of the mesopic band's low end |

## Checked by Alex's Consensus pass (visual polish note section 17.2, `SSE4-visual-polish-citation-check.md` section 15, 24 Nov 2026)

Status as that check gives it. These confirm the stand-in's heights: green 110 km lies in the measured 100 to 115 km,
red 250 km in 150 to 350 km. They are the light thread's to use for MF9 part 2's lifetimes and heights.

| # | Reference | Relied on | Status | Checked |
|---|---|---|---|---|
| AU7 | Kawamura et al. 2020, Earth Planets Space 72. doi:10.1186/s40623-020-01229-8 | oxygen green 557.7 nm, lifetime 0.7 to 0.75 s, peak 100 to 115 km; red 630.0 and 636.4 nm, about 110 s, 150 to 350 km; N₂⁺ 391.4 and 427.8 nm, about 10⁻⁸ s, 100 to 120 km | checked (Consensus pass) | Earth table, section 15 |
| AU8 | Kalogerakis et al. 2009, Ann. Geophys. 27, 2183. doi:10.5194/angeo-27-2183-2009 | the same Earth table | checked (Consensus pass) | as AU7 |
| AU9 | Whiter et al. 2023, Ann. Geophys. 41, 1. doi:10.5194/angeo-41-1-2023 | the same Earth table | checked (Consensus pass) | as AU7 |
| AU10 | Dyudina et al. 2016, Saturn's aurora observed by the Cassini camera at visible wavelengths, Icarus 263, 32. doi:10.1016/j.icarus.2015.05.022 | a hydrogen giant's visible aurora: pink low, purple at 1000 to 1500 km, strong H-alpha (the check against which part 2's hydrogen lines are drawn) | checked, primary | quoted |
| AU11 | Knutsen et al. 2025, Detection of visible-wavelength aurora on Mars, Sci. Adv. 11(20), eads1563. doi:10.1126/sciadv.ads1563 | Mars's green line 101 ± 14 R: a long-exposure subject, near invisible to the Eye exposure | checked, primary | re-read in the pass |
| AU12 | Slanger et al. 2001, Science 291, 463. doi:10.1126/science.291.5503.463; Gray et al. 2014, Icarus 233, 342. doi:10.1016/j.icarus.2014.01.029 | Venus's green line (night airglow; stronger after ejections) | checked (Consensus pass) | section 15 |

Note: the oxygen stand-in draws only where O₂ or O is in the air, so Mars's and Venus's green (from CO₂'s broken
molecules) waits for MF9 part 2's own lines, as rule 5 asks; it is not borrowed.
