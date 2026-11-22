# SOL1: the catalogued Solar System (packs/base-sol)

Asked for by the chart room thread for the start sheet's "Open the Solar System" door, which enters Sol as one
batch with evidence "catalogued". Built in the master data sheet track because the values must be cited, not
typed (MP22).

## Where it lives, and why not on the sheet

The matter-pack design (section 3, "Reference objects") treats the sheet's Sun, Earth, Jupiter and Moon as
yardsticks for units. Changing "1 solar luminosity" changes a unit, not the Sun of a Sol system. So Sol's
bodies go in their own data pack, `packs/base-sol`, under `provides.other.sol`. The sheet's reader is not
involved.

`scripts/build-base-sol.py` writes the pack; its `--check` fails if the pack is stale. The build also checks:
- the Sun's radius, luminosity and temperature equal the sheet's IAU nominal values, and its mass is within
  2e-4 of them;
- Earth's mass and mean radius agree with the sheet's rows, within 2e-4;
- every value's source is in the pack, and every source has a DOI or URL;
- every host is a body in the pack;
- every region has inner < outer.

`scripts/sol_extra.py` holds the values from papers, and reads the astronomical unit from the sheet.
`scripts/check.sh` runs the `--check`. CI does not run it yet: adding it would be a CI change, which needs
Alex's yes. The sse-pack test `the_catalogued_solar_system_pack_loads` loads the manifest and checks its
sources in every Rust run.

## Shape (agreed with the chart room thread)

- `bodies[]` is `{id, name, kind, host, values: {field: {v, unit, src, note?, check?}}}`, with the kernel's
  field names:
  - `body.mass` (kg);
  - `body.radius` (m, volumetric mean);
  - `orbit.semi_major_axis` (m);
  - `orbit.eccentricity`;
  - `body.rotation_period` (s, sidereal, always positive);
  - `body.obliquity` (deg, above 90 is retrograde spin);
  - `body.albedo` (Bond);
  - `body.albedo_geometric`;
  - for the Sun, `star.luminosity` (W) and `star.effective_temperature` (K), and `system.age_gyr` (Gyr).

  `host` is another row's id: the Moon's is `earth`, the Sun's is null.
- `regions[]` is `{id, name, host, inner_m, outer_m, src, grade, note, check?}`. It holds the main belt, the
  Kuiper belt and the Oort cloud.
- `sources[]` uses the sheet's source shape.

## Sources

- The Sun, the eight planets, the Moon and Pluto come from the NASA NSSDCA fact sheets, read 22 Nov 2026 and
  copied as printed, then turned into SI. Jupiter's and Saturn's rotation are System III; Uranus's and
  Neptune's are from Voyager radio science.
- The Sun's age is 4.5673 Gyr, the age of the first solids (Connelly et al. 2012).
- Ceres (from Dawn):
  - radius 469.7 km, mass 9.384e20 kg and obliquity 4° (Russell et al. 2016, with Park et al. 2016 beside);
  - geometric albedo 0.094 (Ciarniello et al. 2017);
  - orbit and rotation from the NSSDCA asteroid sheet;
  - "Bond" albedo 0.034 from Li et al. 2016, marked unverified: the text calls it the Ceres average, and
    whether it is a Bond albedo was not confirmed.
- The main belt runs between the 4:1 and 2:1 resonances with Jupiter, worked out from Jupiter's semi-major
  axis by Kepler's third law (2.06 to 3.28 AU). Grade: convention.
- The Kuiper belt runs from Neptune's orbit (a convention) to the classical belt's outer edge at 50 AU
  (Trujillo, Jewitt and Luu 2001).
- The Oort cloud runs from 5,000 to 100,000 AU and is graded an estimate. The outer limit, 1 to 2 × 10^5 au,
  set by the Galactic tide, is quoted from Kaib and Volk 2022. The inner edge was read in paraphrase, so it
  is marked unverified. It is a sphere (ruling).

## Not yet done

- Moons other than ours, and more dwarf planets.
- Rings.
- Each body's composition or atmosphere: the atlas and the formation pack hold those.
- A journal DOI for Kaib and Volk 2022.

Goldens: none changed.
