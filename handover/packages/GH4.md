# GH4: remains as places (streams, shells and mixed remains)

Track GH (`design/SSE4-galaxy-histories-design.md`, section 7.6), built 23 Nov 2026 by the kernel thread. What a galaxy tore apart stays in it as places: its envelope holds each torn satellite's stars as a density, the star cells draw halo stars from them with the ages and metals of the galaxy they were born in, and the sky sees them.

## What it does

### Stars leave by the tidal radius (`infall.rs`, F46)
- Stars outside the tidal radius now leave at the same rate as the dark matter outside it.
- The share inside is that of an exponential disc of the satellite's own scale: 1 − (1 + x) e^(−x), with x = r_t / R_d.
- A star once lost never comes back.
- This replaces F46's line "stars stay until a tenth of the dark matter is left". Stars deep in the dark halo still go late (Peñarrubia et al. 2008).
- The GH2 check now reads:
  - stars kept ≥ dark matter kept;
  - with half the dark matter bound, more than four in five stars are.
- Result: Milky-Way-mass stellar haloes are 5 × 10⁷ to 6 × 10⁹ M☉, median about 6 × 10⁸, against the Milky Way's 1.4 × 10⁹. Before, they were 10⁶ to 3 × 10⁸ (GH3's known limit).
- Fate also keeps a finer path, a point every 0.02 Gyr (`path_gyr`), which is not sent.

### Components (`remains.rs`, F49)
- `remains::components` builds one `Component` per member that lost stars. Each holds:
  - the stars lost and when;
  - the loss-weighted nearest and farthest distance, and the period;
  - the spread, (m/M(<r))^(1/3) at its last close pass;
  - its path;
  - its galaxy's births, from its own rings: 4 rings and 40 time bins, each bin with [Fe/H], its spread, [α/Fe], C/O, [r/Fe] and [K-40/Fe].
- `feed::with_remains` puts the components in `Assembled.components`. The kernel's `fed_galaxy` calls it, so the galaxy chart, the home galaxy, the star cells, the sky and origins all have them.
- Shapes at a time:
  - **Stream**: the stars lost less than a mixing time ago, as equal Gaussian blobs along the path. The half length grows as spread × time since the stars left, up to half a period; the width is spread × r (Johnston 1998).
  - **Shells**: the same stars on an orbit with nearest over farthest under 0.1, as a cap at the turning point.
  - **Mixed remains**: stars lost earlier, as a cloud ∝ 1/r² between nearest and farthest, falling off beyond the farthest.
- The mixing time is half a period over the spread, at most GH2's 5 Gyr. A big satellite's broad debris mixes in a few orbits; a thin stream lasts.
- `remains::Cache`, carried on `GalaxyEnvelope.remains`, works the shapes out once for each time.

### Envelope
- `remains_at(place)` gives each component's stars per pc³ at a place, and `remains_pc3(place)` their total.
- `local` counts them:
  - in `stars_pc3` and `systems_pc3`;
  - in the new `remains_pc3` field, sent only when it is not zero;
  - in the encounter rate, at the circular speed, because halo stars barely turn.
- `systems_pc3` itself stays the disc's, so every disc draw is bit-identical.
- The sky's glow adds the remains, lit with the disc's light mix (F49).

### Halo stars (`star_tree.rs`, `stars.rs`)
- A second count tree, `TreeWalk::remains` (`Layer::Remains`), has its own keys: `cosmos.remains.tree.v1` and `cosmos.remains.c<class>.v1`.
- Each star:
  - draws its component by the densities at its cell;
  - draws its birth from that galaxy's births and its mass, on condition of its luminosity class;
  - takes the abundances of its birth bin;
  - gets an orbit with the halo's reach (F49).
- Addresses are `.../sys:f<frame>h<slot>`, and `draw_one` reads them.
- `StarSystem.born_in` names:
  - the galaxy;
  - when it fell in;
  - when it was torn apart or reached the centre;
  - its stars;
  - where the star is (`stream`, `shell` or `mixed remains`);
  - the card's line, for example "born in a dwarf galaxy that fell in 9.6 Gyr ago and was torn apart 7.2 Gyr ago".
- A halo star's formation population is `stellar halo`.
- `Stars::cell`, `cells` (voyages) and `sky_walk` include the halo stars. Pins of each kind take the place of their own kind's fill.
- A galaxy with no components, such as every unfed galaxy and the cosmos crate's scenarios, has no halo tree and gives exactly what it gave before.
- The remains' class grid is kept between walks, so a single cell costs about 5 % more (64 cells near the Sun: 470 ms against 445 ms natively).

### Origin (`origin.rs`)
- `reconstruct` weighs the disc against every swallowed galaxy at the star's place. Each weight is stars per pc³ born within half a Gyr of the birth, times the fit of a kept [Fe/H].
- When a swallowed galaxy wins, the origin names it (`born_in`) and says why. The new reference is Helmi et al. 2018, which is already verified in `docs/citations/galaxy-histories.md`.

## Checks (`crates/sse-cosmos/src/gh4_tests.rs`)
- **Budget**: the components hold the stellar halo to a thousandth, because members under 10³ M☉ lost are left out. The biggest component's density integrates on a grid to within 15 % of its stars.
- **Twelve Milky-Way-mass hosts**:

  | What | Measured | Wanted |
  |---|---|---|
  | Halo mass against the biggest component's stars, Spearman | 0.92 | > 0.6 (Harmsen et al. 2017) |
  | Halo metals against the biggest component's stars, Spearman | 0.97 | > 0.3 (D'Souza and Bell 2018) |
  | Share of the top two components, median | 0.72 to 0.94 (range 0.48 to 1.0) | > 0.6 (Deason, Mao and Wechsler 2016) |
  | Hosts whose halo thins faster beyond the biggest one's farthest reach | 12 of 12 | at least 9 (Deason et al. 2018) |
  | Halo share of stars at the Sun's radius in the plane | 1.8 × 10⁻⁵ to 4.6 × 10⁻³; about 1 in 1,000 for haloes of 1.7 to 3.4 × 10⁹ M☉; rises with halo mass (Spearman 0.92) | 3 × 10⁻⁴ to 3 × 10⁻² for haloes over 10⁹ M☉ |

- **A halo star**:
  - its metals are below the disc stars' near it;
  - its population is `stellar halo`;
  - its card line names the galaxy;
  - `draw_one` gives the same star back;
  - a promoted halo star keeps its slot in its cell.
- **Origin**: very old, metal-poor stars at the Sun's radius fit the stellar halo. For example, 9 to 11 Gyr old at [Fe/H] −2.5 to −1.5 in host x3. A 4.6 Gyr star stays a disc star.

## Goldens re-blessed on purpose
- `tests/golden/contract.jsonl`, 16 lines:
  - the two galaxy charts, with the new components and the halo from the tidal rule;
  - the age-set origins, whose home galaxy changed;
  - the Sun's star cell, the sky and its glow;
  - a voyage plan;
  - a promoted star;
  - the hashes that follow.
- No halo star falls in the contract's own cells.
- The cosmos crate's star, sky and envelope goldens are built on unfed galaxies and do not move.

## Known limits (on F49)
- The halo near the Sun is thinner than the Milky Way's (1 in a few thousand for a 5 × 10⁸ M☉ halo). The debris of the big meals sits close in, because their stars leave near their last pericentres.
- Clouds are spherical, and a stream's blobs carry equal mass.
- Halo orbits use the disc's epicycle form.
- The glow uses the disc's light mix.

## Next
- GH5: globular clusters and every other star cluster.
- Tell the galaxy chart thread that the stellar halo is no longer provisional.
