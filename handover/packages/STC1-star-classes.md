# STC1: every star and remnant has a formal class

Status: built 26 Nov 2026 (thread "Classify every star and remnant"). Alex, 26 Nov: "are stars classified as they go -
formal types classification GIII FV, etc for EVERY star type and every star remnant? like planets get classifications
written about them?", then "this is general - not just the lifeline". Before this, no star had a spectral type or
luminosity class anywhere but V3 import tags, the Traveller reader and the star chooser's own temperature rule.

Ruling kept: processes, not classes. The class is a label read off the life line's results; nothing reads it back.

## What it does

- **Kernel read** `sse_formation::spectral::classify(teff_k, mass_msun, mass_kg, radius_m, phase) -> StarClass` with `code`
  ("G2 V", "K5 III", "B8 Ia", "T5", "DX 4.5", "DC 10", "NS", "BH"), `letter`, `subclass`, `luminosity`, `family`,
  `kind`, `why` (in words and numbers) and `notes` (what it cannot say).
  - Letter and subclass: surface temperature, nearest subclass on the dwarf scale (Pecaut and Mamajek 2013; L, T, Y
    with Kirkpatrick et al. 2019). Giants and supergiants of G, K and M read a giant scale (Allen's, van Belle 1999).
  - Luminosity class: log g from mass and radius against the loci of V, IV, III, II, Ib, Iab, Ia at that temperature
    (Straizys and Kuriliene 1981; Gray and Corbally 2009); the nearest locus wins.
  - Families (the atlas base classes, ids agreed with the class gallery thread): pre-main-sequence, main-sequence,
    subgiant, giant, bright-giant, supergiant, brown-dwarf, white-dwarf, neutron-star, black-hole. A pre-main-sequence
    or brown dwarf phase sets the family; otherwise the luminosity class does.
  - White dwarfs: D, a letter, the temperature index 50400 / T (Sion et al. 1983). Below about 5000 K the spectrum is
    featureless whatever the atmosphere (DC). Hotter, DA versus DB needs the hydrogen layer, which the life line does
    not track, so the letter is X and the class says why.
  - Neutron stars: NS. Pulsar or magnetar needs spin and field (exotic objects packages, XO10 on); room is left.
  - Black holes: BH (under 100 solar masses), IMBH, SMBH by mass.
  - A row with no phase (a catalogued star such as the Sun) is read by gravity alone; white-dwarf gravity (log g over
    6.5) reads as a white dwarf.
- **Published on every star at the date read**: `resolved_system` adds `star.spectral_class`, `star.spectral_letter`,
  `star.luminosity_class`, `star.class_family`, `star.class_kind`, `star.class_why` to every star row (catalogued,
  grown, imported, aged by a look-ahead). A kept fact of the same name wins. As the clock moves, the class moves with
  the star (test `a_class_follows_a_star_through_its_life`: the Sun from pre-main-sequence through G V to white dwarf).
- **describe_star** `now.class` (the same StarClass).
- **Atlas** `packs/base-atlas`, scheme `star` ("What kind of star is this?"), 61 classes: `star/<family>` with
  `star/<family>/<kind>` beneath (O to M for the six luminosity families, M L T Y for brown dwarfs, DX and DC, BH IMBH
  SMBH). Axes `star_family` and `star_kind` read the kernel's facts. Each class's first "in" example is a life-line star
  that lands in it (`star.sample_mass_msun`, `star.sample_age_yr`, `star.sample_feh`), from
  `tests/golden/star-classes.json` (written by the test `star_classes_golden`, which also carries the tables, so the
  numbers live once in Rust). Classes no life-line star reaches today say so and have an illustrative example. Real
  stars are examples too (the Sun, Proxima, Sirius A, Arcturus, Rigel, Betelgeuse, Sirius B).
- `class_sample` refuses a star class, pointing at describe_star and the sample star.
- **Screens**: a star's card leads with Class ("G2 V"), its subtitle and Curious sentence say "G2 V main-sequence star"
  (`ui-contract/src/star-class.ts`), the Atlas page's subtitle and player disclosure read the `spectral` field, the
  Classes page lists the star scheme, and the star chooser's Now line and its letters read describe_star's class
  (`letterOf`; the old `spectralLetter` and the letters table in `star-kinds.json` are gone: one door).

## Physics in force (ER1)

- Surface gravity uses the G in force (`sse_core::constants::g()`) on the star's mass and radius in SI; no new sheet
  read. The G2 V anchor of the dwarf scale is the life line's own `TEFF_SUN` (the tracks keep the real Sun on purpose).
  The kernel converts mass to solar masses with its existing `M_SUN_KG`. The side-door ledger is unchanged.

## Checks

- `sse-formation` `spectral_tests`: the Sun is G2 V; measured stars (Proxima, Alpha Cen B, Sirius A, Aldebaran,
  Rigel, Betelgeuse) within one subclass and the right letter and luminosity class; honest remnant and brown dwarf
  classes; a class through the Sun's whole life; every family reached; the golden `star-classes.json`.
- `sse-sift` `the_star_scheme_reads_the_kernels_star_class`; base atlas test follows the fifth scheme.
- `sse-kernel` `a_star_class_is_drawn_from_the_life_line_not_sampled`; `the_whole_atlas_samples` samples worlds only.
- `ui-contract` classes.test (five schemes, 130 classes, a star class's sample), star-chooser.test (letters are the
  kernel's class).
- Goldens: `classify.json` (atlas hash), `areas/kernel.jsonl` (describe_star gains `class`), core `contract.jsonl`
  one line (the resolved Kestrel star gains its class facts; coordinator told before merge).

- e2e: star-chooser.e2e (Now line reads "main sequence, G2 V, ..."; Red giant "giant, K2 III"), reference.e2e (five
  schemes; the star scheme's tiles).
- Screens: project files `sse4/handover-screens/STC1-screens/` (sun-card desk and phone, star-classes desk and
  phone, star-chooser, star-red-giant).

## Unverified and caveats found

- Every scale and locus is from memory: paper search is out until 1 Dec 2026. `docs/citations.md` rows "Star classes".
- The life line's sketch physics shows through, read honestly: young brown dwarfs read as A to K types (their radius
  is fixed at 0.1 solar radii while the Burrows cooling law is very bright at under a few Myr, so they are far too hot);
  red supergiants read M6 to M9 Ia, cooler than real ones (M0 to M5); a 1 solar mass star near the giant branch tip
  reads bright giant (II). These are life-line causes for the formation owner, not classifier settings.
- No subdwarfs (VI) or hypergiants (0): VI needs a metallicity rule, and neither is given.

## Next

- The star chooser's Kinds chips could come from the atlas star scheme in place of `star-kinds.json` (ED6 note).
- The class gallery ("Build a star class gallery") draws each class from its sample star.
- DA/DB once the white dwarf's hydrogen layer is tracked; pulsar and magnetar once spin and field are (XO).
