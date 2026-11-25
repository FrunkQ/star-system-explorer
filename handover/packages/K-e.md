# K-e: phase tables per kind of particle (light track, 25 Nov 2026)

Visual polish design, kernel ask K-e and VP-A5 (handover/design/SSE4-visual-polish-design.md, sections 12 and 17.1):
the kernel works out how each kind of particle scatters light from its size and refractive index (Mie theory),
once per kind, and every cloud, haze, dust, plume and ring layer reads that table. The Jendersie and d'Eon fit
stays a shortcut for water clouds only.

## What was built

- `crates/sse-atoms/src/mie.rs`: the Mie series for a homogeneous sphere, written fresh from Bohren and Huffman
  1983, with the following pieces:
  - a downward logarithmic derivative;
  - upward Riccati-Bessel functions;
  - aₙ and bₙ;
  - Q_ext, Q_sca and g;
  - S₁ and S₂ at any angles.
  
  It uses its own complex arithmetic through `sse_core::math`, so it is deterministic. The Rust block
  `sse.light.mie_scattering` takes (radius, wavelength, n, k) and gives (Q_ext, Q_sca, g). The atom count goes
  to 139.
- `crates/sse-light/src/scatter.rs`: `Particle {id, radius_m, wavelength_m, n, k, medium_n, spread, angles_deg}` goes
  to `Scattering`, which gives:
  - `q_ext`, `q_sca`, `q_abs` and `single_scattering_albedo`;
  - `g` and `cross_section_m2`;
  - `angles_deg` with `phase`, normalised to a mean of 1 over the sphere.
  
  `spread` is ln σ_g of a log-normal size spread, which smooths the series' ripples.
- `scatter::sheet_index(sheet, substance, "liquid"|"solid", wavelength)`: n and k from the sheet's
  `optical.refractive_index_liquid` / `_solid` tables (water liquid from Segelstein 1981 is on main; ice Ih from
  Warren and Brandt 2008 lands with the data sheet's PR 475). `Particle` may give `substance` and `phase`
  instead of `n`, `k`; `read_bows` items may give `substance` and `wavelength_m` instead of `n` (VP-L4 part 2a).
- Op `read_scattering {items: [Particle]}` returns `scattering_read`. Errors come back as `bad_particle`. It is
  in perf chunk light.

## Checks

- `sse-atoms` `mie_scattering_matches_the_textbook`:
  - Bohren and Huffman's example (n 1.55, x 5.213) gives Q_ext = Q_sca = 3.1054;
  - the Rayleigh limit holds to 0.1 % at x = 0.01, with g ≈ 0;
  - Q_ext comes to 2.00 ± 0.02 at x = 2000;
  - the phase function's mean over the sphere is 1 to 0.1 %, and its mean cosine equals g;
  - a 100 µm water drop in green light peaks at 136° to 141° (the primary bow, which VP-L4 puts at 138°);
  - x = 30 000 is refused, pointing to geometric optics.
- Kernel `read_scattering_gives_phase_tables`: a 10 µm cloud droplet with spread 0.3 gives g 0.8 to 0.9 and an
  albedo of 1, and a 1 m boulder is refused.

- Kernel `read_bows_gives_rainbows_and_halos`: water off the sheet in red light gives a bow of 42.0° to 42.6°;
  `read_scattering` with `substance: "h2o"` gives g 0.85 ± 0.07.

- MF9 part 2b follow-up: each aurora emission carries the light table's `name` (asked by the aurora drawing thread,
  whose renderer may not import packs).

## Golden

`contract.jsonl` changes on purpose:
- the stack hashes and the `reach` list, for the new block;
- two new `read_scattering` cases at the end.

This PR also carries the citation thread's wording fix in `sse-activity`: Skumanich spin falls as the *inverse*
square root of age. No number changes.

## Next

- The sheet's n and k for methane (no clean source yet: Khare's table is OCR-damaged), sulphuric acid, tholin
  haze and dust (data sheet thread); then those substances work by id too.
- Fog bows and glories from these tables (VP-L4 part 2) and the giants thread's haze reading the table.
