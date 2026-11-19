# Citations register

Every reference, published relation and quoted constant in SSE4, with what was checked against the source.
Alex's rule: every block is real, explained physics with a link to its paper (a DOI or URL on every reference,
shown in "Why?"), and a published relation is checked against its source before it is relied on.

## Status words
- **verified**: the DOI or URL resolves to the work named, and the number or relation matches the work
  (units, validity range, equation or table) as used in the code.
- **corrected**: something was wrong and has been fixed. The row says what: *metadata* (wrong DOI, year, page,
  author, equation number; physics unchanged) or *physics* (a misremembered relation or constant; values changed,
  goldens re-baselined with the reason in the package log).
- **unverified**: the source could not be reached or read (paywall, book not online). Not guessed; left as written.
- **textbook**: a standard law (Kepler, Stefan-Boltzmann, Newton) with a textbook pointer; the law itself is checked,
  the page or equation number may be unverified.

## How to add one
Add a row to the area file below in the same PR that adds the reference. Columns: where (file or atom id),
reference as written, what is relied on, status, what was checked.

## Areas
| Area | File | Checked (rows) | Wrong (fixed) | Physics changed |
|---|---|---|---|---|
| Stars | [citations/stars.md](citations/stars.md) | 65 | 5 | multiplicity by primary mass, triple share, main-sequence L and R from the fitted atoms, low-mass q_crit |
| Orbits and formation | [citations/orbits-formation.md](citations/orbits-formation.md) | 101 | 9 | giant-star tide factor (was inverted), pebble isolation mass diffusion term |
| Interiors and heat | [citations/interiors-heat.md](citations/interiors-heat.md) | 32 | 8 | K-40 and U isotope shares, Al-26 decay energy, Zeng radius unit |
| Surfaces | [citations/surfaces.md](citations/surfaces.md) | 72 | 5 | none (wording and DOIs only) |
| Climate and life | [citations/climate-life.md](citations/climate-life.md) | 79 | 9 | methane lifetime in oxic air, hydrogen escape coefficient |
| Galaxy and cosmology | [citations/galaxy-cosmology.md](citations/galaxy-cosmology.md) | 90 | 11 | merger mass ratios, disc heating exponent, Hawking lifetime |
| Light, reference objects and design files | [citations/light-reference.md](citations/light-reference.md) | 108 | 18 | planet disc brightness factor, Sizes tab star radii and temperatures |

Counts are table rows in each area file after the first pass (19 Nov 2026). About a third of rows are still
unverified, mostly because sources could not be fetched in that pass; they say so and nothing was guessed.
