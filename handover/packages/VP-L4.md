# VP-L4: rainbow and halo angles from a refractive index

Light track, 24 Nov 2026 (visual polish design section 12, accepted). The renderer draws the bows;
these are the angles.

- Two Equations in `packs/base-physics/formulas/light.json`:
  - `sse.light.rainbow`: Descartes' minimum-deviation ray after one and two internal reflections
    (Nussenzveig 1977, doi 10.1038/scientificamerican0477-116), written with atan2 because the formula
    language has no asin or acos.
  - `sse.light.halo`: a prism's minimum deviation (Tape 1994, doi 10.1029/AR064).
- `sse_light::bows::bows(id, n, kind)` with `kind` "drop" (primary and secondary bow) or "crystal"
  (halos through 60° and 90° faces).
- Op `read_bows {items: [{id, n, kind}]}` (Cost::Trivial); errors are `bad_bow`.
- Checks: water (n 1.333) gives 42.1° and 50.9°; methane rain (n about 1.29) gives a bow near 49°, which
  confirms the design's figure; ice (1.31) gives 21.8° and 45.7°. Fudge F58.

Next: refractive index rows on the data sheet for the liquids and ices that rain or float (water,
methane, ethane, ammonia, sulphuric acid, CO2 ice), so a world's weather can name its drops by id. The
same rows feed K-e's Mie phase tables.
