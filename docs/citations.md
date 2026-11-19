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
| Area | File | Checked | Wrong | Physics changed |
|---|---|---|---|---|
| Stars | [citations/stars.md](citations/stars.md) | in progress | | |
| Orbits and formation | [citations/orbits-formation.md](citations/orbits-formation.md) | in progress | | |
| Interiors and heat | [citations/interiors-heat.md](citations/interiors-heat.md) | in progress | | |
| Surfaces | [citations/surfaces.md](citations/surfaces.md) | in progress | | |
| Climate and life | [citations/climate-life.md](citations/climate-life.md) | in progress | | |
| Galaxy and cosmology | [citations/galaxy-cosmology.md](citations/galaxy-cosmology.md) | in progress | | |
| Light, reference objects and design files | [citations/light-reference.md](citations/light-reference.md) | in progress | | |
