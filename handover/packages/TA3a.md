# TA3a: giants in colour, and their tiles no longer drawn as ground

Track TA (air in True), owner thread "Build giant bands, clouds and hazes". A drawing-only package that does not
change the golden. It needs the climate thread's PR 418 (W10a, the giant chromophore aerosol) to show its colour.

## Why
Alex, 25 Nov: "saturn looks very black and white". Three causes:
1. **Giants drawn over a grey ground.** `surface_tiles` gives a giant's cloud-deck relief (regime kind `cloud-deck`).
   `readTrueFace` took those tiles as ground, so the decks were laid over grey heights. The session's `surfaceTiles`
   now also returns the world's outermost layer, `kind`. A `cloud-deck` world has no ground, so its face is its decks
   alone (`trueGiant`).
2. **Black below the decks.** `trueGiant` drew the deep air as black. A deep air of conservative Rayleigh scatterers
   over thick deeper decks returns the light that reaches it, so it is now white (`DEEP_AIR`). Its methane darkening
   waits for the light thread's PR 406 (TA4).
3. **No colour.** Weather aerosols only added a tinted reflection. Each aerosol's colour now also counts as its
   single-scattering albedo per channel. It absorbs τ (1 - colour) per channel, and diffuse light crosses it 1.66
   times thicker each way (`addAbsorbing`). Jupiter's chromophore (418) then takes the blue out over its belts, and
   Titan's haze turns orange.

## Seen (local build with 418 merged)
- Jupiter: cream zones and brown belts. Screens are in project files `sse4/handover-screens/TA3a-screens/`.
- Saturn: pale in the first local build, because the weather op built the giants' conditions as nitrogen air, which
  gave Saturn a methane-ice top deck and no chromophore. The climate thread fixed this in 418: giants read their own
  air (a protosolar H2/He mix when none is measured) and the Sol giants carry their 1 bar temperatures. So Saturn gets
  an ammonia deck and the chromophore.
- Uranus and Neptune: white, until methane absorption (406, TA4) gives them their blue-green.

## Records
- Fudge row "Haze that absorbs, and a giant's deep air (TA3a)". Citation: Goody and Yung 1989 (diffusivity),
  textbook.
- Tests in `packages/renderer/test/true-air.test.ts`: belts browner than zones; deep air bright; an orange haze takes
  out blue; a `cloud-deck` world has no ground and comes out brown.

## Next
TA2 then TA3 replace the colour-as-albedo stand-in with each particle's absorption from its refractive index on the
data sheet. TA4 adds the gases' absorption.
