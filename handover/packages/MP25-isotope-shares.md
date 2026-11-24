# MP25: protosolar isotope shares on the isotope rows

The kernel thread asked for this on 24 Nov 2026 for MP8c (giant envelopes), and TG8's "helium-3 on all four giants" depends on it. Each stable isotope row on the sheet now carries `abundance_protosolar`: its share of its element by number, 4.567 Ga ago, as `{v, unit: "atom fraction of the element", src: "lodders2021", check: "unverified", note}`.

- **Source.** Lodders 2021, Space Sci. Rev. 217, 44, Table 4, column 4 ("Isotopic composition atom %"). It was read from the NSF public-access copy (https://par.nsf.gov/servlets/purl/10642789). Lodders takes most compositions from Meija et al. 2016, and uses the Sun's own composition where it differs from Earth's (C, N, O, Ar).
- **Script.** `scripts/isotope_shares.py` holds the percentages exactly as printed and divides them by 100. It also checks that one element's shares never pass the whole.
- **Rows filled.** h1, h2, he3, he4, li6, li7, c12, c13, n14, n15, o16 and o18.
  - The He-3 to He-4 ratio comes out at 1.66e-4.
  - D is 1.97e-5 of hydrogen. That is the protosolar value: Lodders notes that deuterium is "essentially absent in the sun" today.
- **Waiting for rows.** Be-9, B-10, B-11, O-17, F-19, Ne-20, Ne-21 and Ne-22 have shares in the script but no isotope row yet, because their AME2020 masses have not been fetched. They are added once those rows exist.
- **Test.** `protosolar_isotope_shares_are_on_the_isotope_rows`.
- **Golden.** No golden changes.
