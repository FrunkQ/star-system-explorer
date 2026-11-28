# Star close-up: fixes from the GPU check (28 Nov 2026)

Thread "Draw binary mass transfer and flares". On 28 Nov, Alex's PC (WebGPU on the RTX 4070) ran current main. The check is in project files on the PC: `sse4/handover-screens/star-close-up-screens/gpu-28nov/` (screenshots and notes.md; the scripts are in `C:\Development\sse4-sg6-capture\gpu28`).

## Found, and what was done

1. **The Sun's system ran at 3 to 10 fps with the clock running.** System3D's draw effect redrew the star's face and glow on the CPU on every frame. At the close-up that took 133 ms each time (512 px face, 640 px glow).
   - System3D now redraws them only when the star's picture time (ten a second), their sizes, the angle the disc fills (to a thousandth) or the star read changes.
   - The glow is kept per zoom and light, and copied when unchanged (`star-glow.ts`).
   - The sRGB curve is read from a table (`srgb.ts`), not a power per channel per pixel.
   - `drawStar` no longer makes arrays per pixel.
   - Result: 133 ms down to about 30 ms per picture at the close-up (node, one core), at most ten a second.
2. **The last system's star stood in for a few seconds.** SystemChart kept the old `liveStar` while the new star was read. It now draws the star plain until its facts come.
3. **The limb was stepped.** A round limb is now cut on the screen by the GPU (a `SpriteNodeMaterial` whose opacity is the disc's edge, with `fwidth`), so it is smooth at any zoom. A lumpy giant's limb keeps the picture's edge. Clear texels carry the limb's colour, so filtering never darkens the edge.
4. **The limb darkening was weak.** Each colour now darkens by Eddington-Barbier in the same grey atmosphere (`limbDarkeningAt`). The Sun's green limb is 0.33 of its centre and blue falls further than red. It follows the star's temperature. This is drawing-side physics, logged in the fudge register to move into a kernel table through the bake seam.
5. **The gap at the limb does not show.** Closed.
6. **The eruption does not look like one.** Eruptions do lift off and balloon (checked frame by frame). At 400 km/s one leaves the close-up's picture in about half an hour, so stepping by the hour misses it.
   - The words now give its speed and how long it stays in view.
   - A risen rope's thinning goes through the hydrogen-alpha log scale, which is its surface brightness, so it stays visible as it rises.
7. **A flare shows nothing on the disc.** This is physics: white-light flares raise the Sun's light by a few per cent over a small patch. The X-ray band shows them. No change.
8. **The close-up framing.** At 2.6 radii the disc filled the whole height of the view. The camera now stops at 4.3 radii (`STAR_CLOSE_SHARE` 3.3), where the disc fills three fifths of the height and the prominences are in sight.

## Checks

- `packages/renderer/test/star-gpu-check.test.ts`:
  - the sRGB table against the curve;
  - limb darkening by colour and temperature;
  - a kept glow equals a fresh one;
  - the Sun's face is round;
  - the eruption words;
  - a risen rope's light. Seen red with the fade laid over the scale: ratio 108 against 186.
- `close-up.test.ts` updated for the new floor.
- `npm run ui:test` (1233 pass) and `ui:check` are clean.
- Owed: a second GPU look on Alex's PC.
