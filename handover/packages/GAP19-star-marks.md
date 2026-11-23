# V3 gap 19: stars that show what they do (part 1)

Status: part 1 done, 23 Nov 2026 (galaxy charts and travel screens thread). From `SSE4-v3-gaps.md` item 19: V3 drew
flares, jets and shed shells on its star glyphs and sized them by class. In SSE4 the chart's symbology reads the
kernel's facts instead.

## What it does
- **Size by power.** A star's mark size is bound to `number:luminosity_lsun`, in both base symbologies (and so in
  mixed). The thresholds are 0.01, 0.3, 3, 30 and 1000 Suns, giving radii 4.5, 5.5, 7, 8.5, 10 and 12. The Sun stays
  at 7, the size every star had before. With no facts, a star keeps 7.
- **Flares.** A new channel, `aura` (`none`, `flares`, `shell`), is bound from `tag:activity:`. A saturated or active
  corona draws flares: a short kinked stroke on each diagonal, out past the star's rays. The Sun is `moderate`, so it
  draws none. A symbology may bind `aura` from any source; `checkSymbology` refuses values the chart cannot draw.
- **The facts.** `ui-contract/src/star-marks.ts` holds `readStarFacts(session, system, stars)`. It makes one
  `resolved_system` call and one `read_activity` call for all of a system's stars. A star's power comes from
  `star.luminosity`, or from its radius and temperature (Stefan-Boltzmann, constants from the sheet). The shell reads
  the facts once per system and adds them to each star's mark subject (`withStarFacts`). A star the kernel cannot
  describe (a fixture star such as Kestrel's) keeps the plain mark.
- `Mark.svelte` draws the aura; `sameLook` counts it.

## Checks
- `star-marks.test.ts`:
  - the Sun is one Sun;
  - in all three shipped symbologies, the size follows power and only a busy corona flares;
  - a planet never takes an aura;
  - a bad aura is refused;
  - a system's stars cost one `resolved_system` and one `read_activity`.
- In the browser, the Solar System's Sun reads `activity:moderate` and one Sun, so it draws at 7 with no aura.
- ui:check, ui:test, and the shell, symbols, start and styles e2e all pass. No kernel change and no goldens.

## Honest limits and next
- **Shed shells (`shell`)** can be drawn, but nothing binds it yet. A star's planetary nebula needs its life line (the
  `planetary_nebula` event) fed to `read_nebulae`; that is part 2.
- **Jets** are not drawn. The kernel computes no jets yet, and SSE4 draws only what the physics says.
- **Not checked in a browser:** no star in the shipped scenes has a saturated corona, so no browser check has drawn
  flares on one. The unit tests cover it.
- Stars have no legend lines yet (the system chart's key lists planets).
