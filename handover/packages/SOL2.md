# SOL2 Sol's surfaces as catalogued facts

Asked for by the proven core thread (test drive, 23 Nov): with no air or water
facts, Sol's Earth resolved at 255 K, airless and dry. The surface chain now
takes a catalogued world's own values before any estimate, under these keys.

## What landed (`scripts/build-base-sol.py`, `SURFACE`)

From the NSSDCA fact sheets, read 23 Nov 2026; only a printed average is kept.

| Body | planet.surface_temperature | atmosphere.surface_pressure | atmosphere.main |
|---|---|---|---|
| Mercury | 440 K | none (exosphere, printed as an upper bound) | none |
| Venus | 737 K | 92 bar = 9.2e6 Pa | co2 |
| Earth | 288 K | 1014 mb = 101400 Pa | n2 |
| Mars | ~214 K | 6.36 mb = 636 Pa at mean radius | co2 |
| Pluto | none (printed as 24 to 38 K) | ~13 microbar = 1.3 Pa | n2 |
| Moon | none (printed as a range) | none (exosphere) | none |

- `body.surface_water_oceans` 1 for Earth, by definition of the unit (one
  Earth ocean, 1.335e18 m3, Charette and Smith 2010; unverified until the
  citation check reads it).
- `atmosphere.main` is a substance id on the master data sheet, unit
  `substance`; the build checks the id is on the sheet. The composition by
  volume is in the value's note.
- `packages/ui-contract/src/sol.ts`: a value may be a string (only for unit
  `substance`); the door passes it through as a text fact, as the kernel's
  `atmosphere.main` already is.

## Open

- `body.obliquity` stays in deg as printed, with its unit; the door should
  convert by unit (told the proven core thread). If the threads settle on rad,
  change the pack.
