# Corrections from Consensus batch 1 (scored 25 Nov 2026)

Each fix goes to the thread that owns the file. Tick it off in that thread's PR and in docs/citations.md.

## Wrong wording or attribution (must fix)
1. **Skumanich 1972** (crates/sse-activity/src/lib.rs; nebulae-and-light design 4A): rotation and Ca II emission fall as the **inverse** square root of age, not "as the square root of age". The law assumes a Hyades age of 0.4 Gyr.
2. ~~LaRowe and Amend 2015~~: no fix needed. SSE4 already cites LaRowe first; the list had keyed it wrongly (fixed in the gatherer).
3. **Bromm and Loeb 2003** (crates/sse-cosmos/src/nucleus.rs, packs/base-cosmology): the paper's seed mass is 10⁶ M☉ or more (about 5 × 10⁶ condenses), not "about 1e5 M☉". Either change the seed mass or cite another source for 1e5. Doi 10.1086/377529.
4. **Agnor and Hamilton 2006** (exogeology gaps design 2.2, 3.1): the paper covers Triton's capture only. Cite Correia 2009 for the tidal evolution after capture, or McKinnon 1984 for a semi-molten Triton.

## Add a missing detail
5. Boylan-Kolchin, Ma and Quataert 2008 (crates/sse-atoms/src/reach.rs): add the journal (MNRAS) and doi 10.1111/j.1365-2966.2007.12530.x.
6. Heitz and Neyret 2018 (visual polish design 4.2): add the venue (Proceedings of the ACM on Computer Graphics and Interactive Techniques) and doi 10.1145/3233304. It works for stochastic textures only (moss, sand, granite).
7. Frankel et al. 2018 (crates/sse-cosmos/src/origin.rs, base-cosmology): the title is "Measuring Radial Orbit Migration in the Galactic Disk". Our string is a description of the paper, not its title.
8. Aguado-Agelet et al. 2026 (galaxy histories design 7.2): now in A&A, doi 10.1051/0004-6361/202661469. The arXiv id 2608.27611 is unconfirmed.
9. Byrne and Ingersoll 2003 and Thomas et al. 2000 (exogeology gaps 3.2): neither gives the "metres a year" retreat. Cite Malin, Caplinger and Davis 2001, Science, doi 10.1126/science.1066416 (1 to 3 m per Mars year).
10. Narayan and Yi 1995 (nucleus.rs, base-cosmology): the critical rate is about α² Eddington. "About 1%" assumes α ≈ 0.1, so state the α used.
11. Yuan and Narayan 2014 (base-cosmology, exotic objects design 8): an exact "L ∝ ṁ²" holds only for weak electron heating. Credit L ∝ ṁ² to Narayan and Yi 1995.
12. Brueshaber et al. 2019 (crates/sse-weather/src/giants.rs, base-weather): the paper's phrase is "large cyclonic polar vortex", not "diffuse cap". Brueshaber and Sayanagi 2021 add that Bu alone does not separate the Saturn and ice-giant regimes.
13. Hills 1981 (crates/sse-cosmos/src/events.rs): the 10⁴ au in the paper is the comets' semimajor axis. "Passages inside about 10,000 au" is our reading, not a threshold the paper states.
14. Górski et al. 2005 (crates/sse-sky/src/healpix.rs): "figure 3 layout" is unconfirmed. In the preprint the grid is figure 4 and the numbering schemes are figure 5.
15. Perlin and Neyret 2001 (visual polish design): "two extra texture reads" is not from this paper (flow noise is procedural). Check it against Vlachos 2010.
16. Tessendorf (visual polish design): Consensus lists only the 2004 notes. Check the year of the copy used.

## Not reachable through Consensus (another route needed)
- Shakura and Sunyaev 1973: the citation matches, but no text is in Consensus, so the T⁴ law is unchecked.
- Li et al. 2011: the Ia-only Milky Way rate of 0.54 per century is not in the abstract.
- Hills 1975: the Hills mass is not in the abstract.
- Lovis and Fischer 2010 (book chapter) and Kaye and Laby (reference book): Consensus does not index them. The CO and SO2 refractivities are still open.
