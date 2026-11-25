# Corrections from Consensus batches 2 to 5 (200 papers, scored 25 Nov 2026)

Scores: see `handover/citations/MASTER.md`. Replies: `results-02.md` to `results-05.md` beside this file.

## A. Fixed by the run-through thread (citation wording only, no number or behaviour)
- Tijhuis et al. 1993: DOI corrected to 10.1002/bit.260420415 (packs/base-life).
- McBride et al. 2009: the comment in crates/sse-cosmos/src/haloes.rs now gives the paper's symbols, (1+z)^β exp(−γ z).
- Oprea et al. 2026: "on average 12%" (observed), and up to 40 km/s in the paper's simulation from both Magellanic Clouds (galaxy histories design).
- The list's own gatherer: SICI DOIs with angle brackets (Bodhaine 1999, Freytag 2002) are now read whole. They were never truncated in SSE4.
packs/base-physics is the frozen copy the contract golden runs on (base_physics_fixture_is_the_real_pack), so its text waits for the golden lane: see B0.

## B0. Citation wording inside packs/base-physics (frozen for the golden; owner: kernel thread, in the golden lane)
- Li et al. 2018: the 7.5 W/m² is Cassini's, not Juno's (flows/body-magnetism.json and pack.json, the Jupiter test name).
- Zeitlin et al. 2013: 1.84 mSv a day was measured inside the spacecraft, behind its shielding (formulas/shield.json, and the copy in crates/sse-atoms/src/test/shield.rs).
- Petrenko and Whitworth 1999: k = 651/T is in W/m/K, not W/m (formulas/heat.json, and the copy in crates/sse-atoms/src/test/layers.rs).
- Batch 1, still open: Skumanich's "inverse square root" (sse-activity), the Frankel title (origin.rs), the Boylan-Kolchin DOI (reach.rs), and Yuan and Narayan (base-cosmology).

## B. For the owning thread: a number, a range or a claim changes
1. **Kesler 2008** (crates/sse-atoms/src/matter.rs, sol_tests.rs, formulas/composition.json, base-surface): 1.7e11 t is porphyry copper at all crustal depths, not above 3.3 km. Drop "above 3.3 km" or recalibrate.
2. **Lada and Lada 2003** (base-cosmology, base-cosmos milky-way-like): M⁻² is measured only from 50 to 1000 M☉, with a drop below 50. "10 to 1e5" is an extrapolation and should be labelled one.
3. **Ojakangas 1989** (formulas/heat.json sse.heat.ice_shell, tests): the paper's bound is 25 km, not "10 to 30 km".
4. **O'Neill et al. 2015** (crates/sse-weather/src/giants.rs rule text, base-weather): the paper predates Juno and does not describe Jupiter's ring of cyclones. Drop it from that rule. Its threshold is deformation radius over planet radius and an energy parameter, not W/m².
5. **Brace and Kliore 1991** (formulas/field.json sse.field.ionopause, tests): ionopause heights are 200 to 300 km at solar minimum and 200 to over 1000 km at maximum (Kliore and Luhmann 1991), not 250 to 400 km.
6. **Showman 2011** (crates/sse-weather/src/formations.rs, base-climate, base-weather): the rising air and the cloud shield come from Yang, Cowan and Abbot 2013, not from this paper.
7. **Kormendy 2013** (crates/sse-cosmos galaxy.rs, base-cosmology): holes track bulges, not whole galaxies. Many bulgeless galaxies host holes of 1e5 to 1e6 M☉.
8. **Smart 2005** (shield tests, magnetic fields design): the 0.39 mSv a year dose is not this paper's subject and needs its own source.
9. **McDonough and Sun 1995** (formulas/composition.json, planet-heat flow): "about 20 TW radiogenic" may not be this paper's figure and needs its own source.
10. **Frank et al. 2014** (crates/sse-atoms/src/heat.rs, planet-heat flow): the abstract says nothing about Mars's volcanism ending 0.5 to 1 Gyr ago. That needs another source.
11. **Antonucci 1993** (base-cosmology): the faithful wording is "orientation explains much, not all". Pure orientation is a "straw person model".
12. **Kruijssen et al. 2020** (galaxy histories design): treats Kraken as accreted. The doubt about it rests on Horta et al. 2021 alone.
13. **Raghavan et al. 2010**: the paper prefers like-mass pairs, so the flat q draw is our simplification (already fudge F-S3).
14. **Blair et al. 2017** (lava tubes): "a roof spans wider as gravity falls" is our inference, not a stated result.
15. **Zurek and Martin 1993**: the causes "dry loose ground, thin air, strong winds" are our gloss.
16. **Globus 2017**: at 4 rpm visitors may still need a day to adapt, and at 6 rpm it is visitors, not residents, who need training.
17. **Shibayama et al. 2013 against Vasilyev et al. 2024**: superflares every 800 to 5000 years against once a century. Say which one is used.
18. **Catling 2017**: the constant cites chapters 2 and 4 and a comment cites chapter 5. Make them agree.
19. **Luhmann 1991**: the constant is named LUH95 for a 1991 paper.

## C. Wrong or doubtful metadata, needing a look at the source
- Gray 2005: Consensus gives 10.1017/CBO9781316036570 for the 3rd edition, not 10.1017/CBO9780511614576 (formulas/gas.json). Check which one resolves.
- Greeley 1971 (crates/sse-surface/src/tubes.rs): the paper described is in the journal now called Earth, Moon, and Planets, not Modern Geology 2, 207. Fix the journal or cite the Modern Geology paper by its own title.
- Glasstone and Dolan 1977: Consensus shows another DOI (the defence-archive copy) and not 10.2172/6852629.
- Wagman et al. 1982: the standard state is 1 bar, and an erratum exists (1989, doi 10.1063/1.555845).
- Thauer et al. 1977 (page 809) and Mastin et al. 2009 (doi 10.1016/j.jvolgeores.2009.10.013) have errata. Check that they don't touch our values.
- Two papers share a key: andrews2013 holds Andrews et al. 2013 (disc masses) and Andrews and Martini 2013 (mass-metallicity). The second was not checked.
- Companion papers cited under one key: Petit, Laskar and Boué 2017 (doi 10.1051/0004-6361/201731196) under laskar2017; Catling and Claire 2005 (doi 10.1016/j.epsl.2005.06.013) under catling2005; Krissansen-Totton et al. 2018 (doi 10.1126/sciadv.aao5747) under krissansentotton2016.
- Mitchell and Vallis 2010: the DOI is JGR Planets.

## D. Missing title, journal or DOI (to add in a later metadata pass; the DOIs are in the results files)
Bruneton 2008, Maehara et al. 2012, Thompson et al. 1987, Cummings et al. 2018, Madau 2014, Mittag et al. 2023, Vasilyev et al. 2024, Nichols 2011, Andrews et al. 2010, Casagrande et al. 2011, Fatuzzo 2008, Hayashi 1981, Lichtenberg et al. 2016, Karkoschka 2010, Owen et al. 2012, Fakhouri et al. 2010, Duchêne and Kraus 2013, Komacek 2016, Sheth 1999, Wordsworth 2015, Haqq-Misra et al. 2018, Kroupa 2001, Eggleton 1983, Chyba 2001, Field et al. 1998, Behroozi et al. 2019, Burrows et al. 2001, Canfield et al. 2006, Forget et al. 2017, Lineweaver et al. 2004, Barkana 2001, Asplund et al. 2009, Lane 2010, Nakamoto 1994, McBride et al. 2009, Gaudi 2012.

## E. Not checkable in Consensus (books, talks, code, or no text held)
Most of the 51 "not found" papers exist and match, but the number we use is inside a paywalled text or a book: Osterbrock, Rybicki and Lightman, Born and Wolf, Turcotte and Schubert, Pierrehumbert, Perryman, Skolnik, Gray 2009, Liou, Petrenko, and others. They need a library check or the new account's full-text tools. Gaudi 2012, Moore et al. 2004, Sánchez-Lavega et al. 2017 and Hollands et al. 2026 were not found at all.
