# MF0: magnetic fields groundwork (references, measured fields, NAV belts)

Track MF, design `handover/design/SSE4-magnetic-fields-design.md` (accepted 22 Nov 2026), section 11.

## What was done

- **References.** All 108 references in design section 14 were checked against a page that prints the DOI next
  to the paper's details. Those pages were publisher pages, NASA ADS, PubMed, Crossref records, institutional
  repositories, and arXiv pages that print the journal DOI. The results are in `docs/citations/magnetic-fields.md`:
  - 104 verified (metadata);
  - one verified from its abstract (Millot 2019, read in MP18);
  - two corrected;
  - one book (Kivelson and Russell 1995, no DOI).

  The corrections, made in the design (repo and project copies) and in the code:
  - P2 Christensen and Aubert 2006: the DOI is 10.1111/j.1365-246X.2006.03009.x, not 02972.x.
    `crates/sse-atoms/src/magnetic.rs` `CA06` now carries the right DOI and the full title. The string is shown in
    "Why?" only, so no golden changed.
  - G1 Beck: the year is 2016 (A&A Rev. 24, 4), not 2015.

  The matter-pack design's section 11 ice references were checked in MP18, in `docs/citations/matter-reference.md`
  section A4.
- **Measured fields.** `tests/fixtures/magnetic/measured-fields.json` is built by `scripts/build-measured-fields.py`
  (`--check` fails if it is stale). It holds 28 bodies, each with its source (DOI or URL) and a grade:
  - the eight planets, from the NSSDCA fact sheets: dipole at the reference radius, tilt and offset, with the
    moment worked out as m = B0 R³ × 1e7. Earth comes to 7.94e22 A m², against IGRF's 7.7e22;
  - Venus and Mars, with no global dipole; Mars' crustal maximum and largest source (Acuña 1999);
  - Ganymede's intrinsic dipole (Kivelson 2002);
  - Europa's and Callisto's induced fields (Kivelson 2000; Zimmer 2000, the latter as quoted by Liuzzo 2017);
  - the Moon's strongest crustal anomaly (Tsunakawa 2015);
  - the Sun's polar field at minimum and its sunspot fields (Petrie 2015; Okamoto and Sakurai 2018);
  - six M dwarfs with ⟨B⟩ and Rossby number from Reiners et al. 2022 Table B.1, from 0.002 (WX UMa, saturated, 6.9
    kG) to 1.9 (HD 165222, 0.27 kG). Ro is worked out as Per / tau from the table's own columns;
  - two hot magnetic stars: HD 215441 (Babcock 1960, Landstreet 1989) and the O star HD 191612 (Donati 2006);
  - three white dwarfs: WD 2047+372 at 57 kG (Landstreet 2016), Grw +70°8247 at 100 to 320 MG (Ferrario 2015), and
    PG 1031+234 above 500 MG (Schmidt 1986);
  - three pulsars, from the ATNF catalogue's VizieR copy (Crab, Vela, J0437−4715);
  - the magnetar SGR 1806−20 at 2e15 G (Kaspi and Beloborodov 2017).

  Pulsar and magnetar fields are graded "computed", because they are inferred from spin-down.
- **NAV belts from a moment** (the galaxy charts thread, PR 124). `NavBody.magnetic_moment_a_m2`, with belts
  reaching to the Chapman-Ferraro standoff, capped at 15 radii, and a dose scaled by the surface field. Earth and
  Saturn now have belts, and that is tested. Bodies with no known moment keep a 10 Earth-mass fallback, recorded in
  the fudge register, until MF2 computes moments for grown worlds. `kernel/nav.rs` is to fill Sol's moments from
  this fixture.

## Uncertain, and marked so

- Callisto's values are quoted by later papers, not read from Zimmer et al. 2000 itself (`check: "secondary"`).
- Grw +70°8247 has no single field: 100 to 320 MG (Ferrario 2015), 400 MG (Bagnulo and Landstreet 2019). The
  fixture keeps the range.
- HD 215441's "dipole" depends on the model. Babcock's 34.4 kG is the mean surface field. 67 kG is the dipole term
  of a fit that also has quadrupole and octupole terms.
- Pulsar values come from VizieR's 2017 copy of ATNF. The live catalogue may differ in the last digits.
- Earth's tilt (9.35°) is worked out from the dipole pole latitude the fact sheet gives.
- Not found: the Sun's total unsigned flux in Mx, the Pdot of SGR 1806−20, and the Moon's typical crustal field.

## Checks run

- `scripts/build-measured-fields.py --check` passes.
- `scripts/check.sh` passes, and no golden changed.

Goldens: none changed.
