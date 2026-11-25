# Corrections from Consensus batches 6 to 9 (partial replies, scored 25 Nov 2026)

Alex's Consensus plan hit 100% for the month during these batches; it resets 1 December 2026. Papers without a
reply line stay `unchecked` and go out again first. Scores: `handover/citations/MASTER.md`.

## A. Fixed by the run-through thread (citation wording only)
- Dole 1970: the test fixture in packages/ui-contract/test/reference.test.ts now has the real DOI,
  10.1016/0019-1035(70)90095-3.
- Beuthe 2019 and Vallado 2013: SSE4's text was right; the list had put the neighbouring DOI (Čadek 2016, Curtis
  2014) on them. Beuthe rescored confirmed.
- The list's own gatherer: two-letter initials ("Ch.") and two-word co-author surnames ("Correa Barrios") no
  longer make a co-author the key. Tegeler 1999 and Millot 2019 now carry their scores (both confirmed).

## B0. Inside packs/base-physics (frozen for the golden; owner: kernel thread, golden lane)
- Petit, Laskar and Boué 2017 (formulas/stability.json, sse.stability.amd_critical): the DOI cited,
  10.1051/0004-6361/201630022, is Laskar and Petit 2017. This paper's DOI is 10.1051/0004-6361/201731196.
- Gray 2005 (formulas/gas.json): see corrections-02-05 section C.

## B. For the owning thread: a number, a range or a claim changes
1. **Bond et al. 2017** (packs/base-physics star-main-sequence flow, shell reference-objects.json): Sirius A's
   L is 24.74 L☉ in the paper, not 25.4.
2. **Bullock et al. 2001** (crates/sse-cosmos haloes.rs, base-cosmology, galaxy design): 0.035 is the modified
   spin λ′; the classical λ is 0.042. Say which one the code uses.
3. **Reines and Volonteri 2015** (crates/sse-cosmos galaxy.rs, base-cosmology): its hole to stellar mass ratio
   is 2.5e-4, four times below the 1e-3 used; that figure rests on Kormendy and Ho alone.
4. **Bromm and Loeb 2003** (crates/sse-cosmos nucleus.rs, base-cosmology): direct-collapse seeds are 1e6 M☉ or
   more in the paper, not "about 1e5".
5. **Richardson et al. 2014** (base-reference): quote the equation's own 3% for saturated liquid density;
   0.17% is CoolProp's fit.
6. **Park et al. 2016** (sol_tests.rs, base-sol): Ceres separates rock from volatiles, not metal from rock as one
   comment says.
7. **Konôpková et al. 2016** (sse-atoms test magnetic.rs, base-physics dynamo.json, golden lane): measured on
   solid iron, not the liquid alloy the parameter describes.
8. **Zahnle et al. 2003** (sse-surface impacts.rs, base-surface): Europa's surface is "around 50 Myr" here, against
   the "about 60 Myr" credited to Schenk 2004 (not checked).
9. **Tegmark et al. 1997** (g1_tests.rs, base-cosmology): "3-sigma" is not in the abstract, and its 1e6 M☉ is a
   baryonic mass at z about 15, not a halo mass at z 20 to 30.
10. **Takai et al. 2008** (sse-life traits.rs, base-life): 122 °C holds only under 20 MPa; the low limit near
    253 K is Clarke et al. 2013, not this paper.
11. **Kesler and Wilkinson 2008, Lada and Lada 2003, Skumanich 1972, Greeley 1971, Agnor and Hamilton 2006,
    Tijhuis 1993**: already in corrections-01 and corrections-02-05.

## D. Missing title, journal or DOI (later metadata pass)
Bullock et al. 2001, Konôpková et al. 2016, Tegmark et al. 1997, Takai et al. 2008.
