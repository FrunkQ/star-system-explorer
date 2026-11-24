# WBH part 1: the extended world profile, read off the physics

Alex, 23 Nov: Traveller world making the World Builder's Handbook way, shipped as one "big block" a GM can grab and
change. Rulings (all yes, 23 Nov 22:11): port ideas, not code; keep WBH tables out and use Cepheus Engine SRD (OGL)
numbers for the basic social rolls; never roll a physical digit; the physical profile first, then the social layer;
credit Roy Martin's TravellerSystemGenerator according to its licence (GPL-3.0).

## What changed

- **`sse-codes` gains a `value` segment source**: `{from: "value", field, times?, per?, digits?}` reads a fact,
  multiplies it by `times`, divides it by `per` (a constant on the master data sheet by id: `g0`, `atm`, `au`,
  `M_earth`, `yr_julian`...), and shows it to `digits` significant figures (`sig_figures`). The why names the fact
  and the constant. A missing fact reads `?`. `check` refuses a `per` that is not on the sheet. Value segments are
  read-only: they are never pins for planning a system from codes.
- **A second Traveller code sheet, `traveller.extended`** ("Extended world profile"): twelve lines read off the
  world's own facts, with none rolled: diameter, mass, density, gravity, escape velocity, surface pressure, oxygen
  pressure, surface liquid, temperature, distance, eccentricity and year. It has four layouts: size, air, surface
  and orbit. It is the block: swap it in a pack or edit a line. Earth reads "12740 km across, 1.00 Earth masses,
  5.51 g/cm³, 1.0 g, escape 11.2 km/s".
- **World codes picks a sheet.** When the stack gives more than one sheet, a "Read as" picker sits on the World codes
  panel. The why of a value segment reads "Read off the physics: ... Never rolled." (`sheetsOf`, `whySentence`).
- **Credits**: `provides.other.credits` and a Credits section in `packs/traveller/LICENCE.md`, for the Cepheus Engine
  SRD and TravellerSystemGenerator (ideas only; no code or tables).

## Checks

- `sse-codes` `the_extended_profile_reads_off_the_physics` (Earth's lines, a missing fact reads `?`, a bad
  constant is refused) and `significant_figures_read_plainly` (including 0.99997 to "1.00").
- `tests/ui/world-codes.test.ts`: two sheets in stack order, and the value why sentences.

Golden changed on purpose: only the pack hash in `tests/golden/traveller-codes.json`. No reading changes.

## Next (part 2, the social layer)

- A seeded, GM-only social layer: government, law and tech level from population with Cepheus Engine SRD numbers,
  drawn deterministically from the world's seed (same on every device), written as authored facts the GM can
  overrule, and marked as rolled. Physical digits stay read off physics.
- Day length and axial tilt once those facts exist on every world; tidal and seismic lines from the kernel's tides.
