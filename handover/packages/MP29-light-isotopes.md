# MP29: the rest of the stable isotopes from H to Ne

MP25 put Lodders 2021's protosolar isotope shares (Table 4) on the isotope rows the sheet already had. It could not place shares for Be-9, B-10, B-11, O-17, F-19, Ne-20, Ne-21 and Ne-22, because those isotopes had no rows yet. This package adds those eight rows, each with its relative atomic mass. With them in place, every share in Lodders' table has a home, and the builder now asserts that no share is left over. p-B11 fusion and the neon isotope ratios in giant envelopes can read these rows.

**Masses.** The masses come from NIST's Atomic Weights and Isotopic Compositions pages, read on 24 Nov 2026 as printed (source `nist-compositions`). Those pages do not say which atomic mass evaluation they use. Their H-1 mass (1.007 825 032 23 u) is one digit off AME2020's in the last place, so these look like AME2016 values. The AME2020 table itself could not be reached from the cloud: the DOI landing page refuses automated readers, and the AMDC hosts are blocked. All eight masses are marked unverified.

**Test.** `protosolar_isotope_shares_are_on_the_isotope_rows` now also checks two things:
- the three neon shares sum to 1;
- B-11/B-10 matches Lodders' 80.173/19.827.
