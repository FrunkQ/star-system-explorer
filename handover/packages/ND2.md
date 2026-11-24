# ND2: light through gas, one mechanism (part 1, with the first rows)

Design: `handover/design/SSE4-new-physics-demo.md` sections 2.4, 2.6 and 3 (ND2 row), accepted 22 Nov.
Owner: Build light and nebulae. Data rows with the matter data sheet thread.

## What landed

- **Row shape, agreed with the data sheet thread (23 Nov).** Each substance's `optical.absorption` is a
  list of features, each with `id`, `src`, `valid_k` `[min, max]`, `note` and `check`, and one of:
  - `table`: `lambda_nm[]`, `sigma {v: [...], unit: "m2"}`, `at_k`, `t_exponent` (σ scales as (T/at_k)^n);
  - `line`: `lambda_nm`, `f`, `lower_ev`, `gamma_rad_s`, `broadening {perturber (sheet id or "any"),
    gamma_per_n {v, unit: "m3 s-1"}, t_exponent, at_k}` (`at_k` added here: the temperature the
    widening was measured at);
  - `pair`: `partner`, `lambda_nm[]`, `k {v: [[...] per temperature], unit: "m5"}`, `at_k[]`.
  An empty list means none known (said); a missing list means not yet looked up (said). No table is
  stretched past its wavelengths (no absorption counted there, said) or temperatures (nearest row, said).
- **Nine formula blocks** `sse.gas.*` in `packs/base-physics/formulas/gas.json`: line strength
  (classical oscillator, Boltzmann share, stimulated emission; e, m_e, ε₀, h from the sheet), line widths
  (thermal Doppler, damping plus widening linear in perturber density), pseudo-Voigt width and shape
  (TCH 1987), line and band cross-sections, optical depth, pair optical depth (HITRAN CIA), transmission
  (Beer-Lambert with a path factor). **Two eye blocks** in `light.json`: `sse.light.eye_lobe` and
  `sse.light.srgb_encode`.
- **The eye as pack data**: `eye` in base-light (CIE 1931 2° lobes after Wyman, Sloan and Shirley 2013,
  380 to 780 nm in 5 nm, XYZ to linear sRGB). `sse_light::colour::colour_of` gives XYZ, chromaticity,
  linear sRGB, sRGB bytes, hex and whether the colour is beyond a screen. This is the kernel's own copy of
  what `packages/ui-contract/src/light.ts` does; screens move over when they read the op.
- **`sse_light::gas`**: `GasOptics::from_sheet`, `Layer`, `Passage`, `pass` (any light, any gas mix,
  any number of layers; nothing names a kind of body).
- **Kernel op `light_through_gas`**: passages of a light (`light` description or `t_k` black body),
  layers and a path factor; returns `passed` (light in and out, optical depth, each feature's peak, what
  was outside the measured range, none known, not looked up, why, stand-ins, references) and
  `colour_in`, `colour_out`.

- **First rows on the sheet** (through `scripts/build-base-reference.py`, with `build-ui-reference.py` and
  `build-rust-reference.py`): methane's visible bands as a `table` (Karkoschka 1994, 410 to 1000 nm, every
  5 nm of the 1 nm table, kept in `handover/packages/ND2-data/ch4-karkoschka1994.csv`); new substance rows
  `na` and `k` (vapours) with their two resonance lines each as `line` rows (NIST Handbook wavelengths and
  Aki; f worked out from Aki and the statistical weights: 0.641, 0.320, 0.668, 0.332). Register rows in
  `docs/citations/matter-reference.md`.

## Tests

- `sse-light` `gas_tests`: black bodies of 5800 K and 10 000 K against the paper's values; a measured band
  only where measured; gases with no rows said clear; an optically thin line keeps its whole strength
  (within 2 %) and a crowding gas widens it; pairs grow with density squared and interpolate in
  temperature; a gas absorbing in the red turns reflected sunlight blue-green; on the real rows, sunlight
  through 3 km-amagat of methane and back comes out pale green-blue and through 30 km-amagat the blue of
  Uranus and Neptune (strongest band in view at 725 nm); sodium's and potassium's D2 lines are twice their D1
  and a thin vapour dims 589 nm and not 550 nm.
- `sse-kernel` `light_through_gas_gives_the_colour_to_the_eye`.
- Golden `contract.jsonl` changes on purpose: the op list gains `light_through_gas` (hello and the unknown
  variant list), two contract cases at the end, the base-physics pack's hash (new blocks) in the stacks
  that load it, and the G reach report lists the new blocks. No world's values changed.
- Fudge F52. Atom count 94 to 105 (P1.md).

## Part 2a (24 Nov): widening from a cited law

- Block `sse.gas.van_der_waals_widening` (gas.json, atom count 122 to 123): Unsöld's C6 =
  (e²/4πε₀) α_p a₀² (5/2) Ry² / h · [1/(I − E_u)² − 1/(I − E_l)²] and the Lindholm width per
  perturber 17 v^(3/5) C6^(2/5), with v the mean relative speed. e, ε₀, a₀, h, c, R∞ and k from the sheet;
  17 is a model number (Gray chapter 11).
- Sheet (through the builder): `optical.polarisability` (m³, NIST CCCBDB) on h2, he, n2, ch4, ar, h2o;
  `optical.ionisation_energy` (eV, NIST Handbook) on na and k. `GasOptics` reads both.
- `pass`: a line with no `broadening` row is widened by every gas in its layer that has a
  polarisability, when the line's own gas has an ionisation energy; the why-lines say so and the
  block's references join the reply. With nothing to work from it says so under `stand_ins`.
- Test `sodium_crowded_by_helium_widens_by_van_der_waals`: C6 by hand; 1 nm off D2 darkens more than
  ten times in 1e27 m⁻³ of helium at 6000 K. Fudge F54 (Unsöld is often about two times narrow).
- No golden change beyond the base-physics and base-reference pack hashes.

## Part 2b (24 Nov): the limb law per colour

- Two Equations in `packs/base-physics/formulas/light.json`: `sse.light.limb_temperature`
  (Eddington-Barbier in a grey atmosphere, T = Teff (¾(μ + ⅔))^¼; Rybicki and Lightman 1979 §1.8,
  doi 10.1002/9783527618170) and `sse.light.reflected_limb` (Minnaert 1941, b = μ0^k μ^(k−1),
  doi 10.1086/144279). New kinds `light.limb_cosine`, `light.seen_temperature`, `light.minnaert_k`,
  `light.incidence_cosine`, `light.brightness_share`.
- `sse_light::limb`: `glowing(teff, mus, wavelengths, eye)` gives each point's layer temperature,
  brightness per wavelength against the centre (Planck at that layer over Planck at μ = 1) and the
  colour to the eye; `lit(k, mu0, mus)` gives Minnaert's share per k.
- Op `read_limb` (Cost::Trivial, light chunk): a disc gives `teff_k` (glowing) or `minnaert_k` and
  `mu0` (lit); wavelengths default to the eye's grid. Errors are `bad_disc` with the disc's number.
- Tests: Eddington's (2 + 3μ)/5 law; blue darkens more than red and the edge is redder; Lambert at
  k = 1 and a brighter edge for k < 1; kernel `read_limb_gives_the_edge_of_a_disc`. Fudge F55.
- Golden change: the base-physics pack hash and the contract cases for the new op only.

## Next (part 2)

- More rows: H₂-H₂ and H₂-He pairs (HITRAN CIA; hitran.org was not reachable from the build machine), a cited
  measured widening rows where they exist (Allard et al., ABO), which win over Unsöld's estimate, methane below 410 nm and in the near
  infrared, TiO and C₂ substance rows, ammonia and the giants' chromophores.
- A spectrum on a wire (a `list<number>` on the eye's or a named grid) and the gas block in flows.
- The light of a description's lines in the colour; screens read colour from the op.
- Gap items 9 and 36 build on it (own daylight, gas giant colours from cloud chemistry).
