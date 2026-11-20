# R2a: air from the kernel's facts, physical lighting, gpu-high, "True colour"

Status: done (19 Nov 2026), renderer track, PR into `main` after R1 (PR 20).
Authority: `design/SSE4-surfaces-and-rendering.md` 6.1 (gpu-high), 7 (Hillaire atmosphere recommended), 8.6; surface design section 9 (R2 row). R2 is split into small PRs: R2a (this), R2b clouds and finer tiles, R2c magnetic field overlay, R2d descent views.

## What was built (three lines)
1. `atmosphere.ts`: a world's air from kernel facts (surface pressure, surface temperature, gravity, gas mix): Rayleigh scattering per gas after Bucholtz (1995), number density from P/kT, scale height kT/(mu m_u g), at 680, 550 and 440 nm.
2. `gpu-air.ts`: an `atmosphere` pass that ray-marches single scattering through a shell around the globe (Nishita et al. 1993), with the planet's shadow; `gpu-globe.ts` gains physical lighting (linear albedo from the kernel's albedo layer, sun transmittance by the Kasten and Young (1989) air mass, ACES tone mapping); the canvas tier gets a one-column approximation.
3. The `gpu-high` tier (chosen by itself only on WebGPU, available on request on WebGL2) and the "True colour" recipe (gpu-high, gpu-basic, canvas, text); the badge says where the air came from ("Air from the kernel: 1013 hPa at 285 K, N2 78%, O2 21%; scale height 8.x km; blue light's path straight up scatters 2x%").

## Design sections and deviations
- Hillaire (2020) is the recommended atmosphere. This package does Nishita-style single scattering first, because it is exact for thin air and has no lookup tables to build; Hillaire's multiple scattering and transmittance LUTs are a later step (listed as a fudge below). No ADR: the report names Hillaire as the target, not a ban on single scattering.
- Physical lighting is a `lighting` parameter of the relief pass (display or physical), so map recipes keep tier parity and the photographic look is opt-in.

## Commands
```
npm run ui:test     # renderer/test/atmosphere.test.ts among them
scripts/build-shell.sh && node tests/ui/render.e2e.mjs chromium --screens <dir>
```

## Results
| Check | Result |
|---|---|
| Earth's air from its facts matches Bucholtz's sea-level Rayleigh coefficients | 4.88, 11.40, 27.8 per million metres against 4.84, 11.49, 28.6 (within 3%); scale height 8.43 km |
| Mars-like and Titan-like air from their own facts | Mars (610 Pa, 210 K, CO2) scale height within 8% of 10.8 km, blue zenith depth under 0.02; Titan (146.7 kPa, 94 K, N2 with CH4) scale height 15 to 25 km, blue zenith column optically thick |
| No air, unknown gases, missing facts | Said, never invented (null air; unknown gases listed with their share) |
| True colour draws on gpu-high and canvas from the kernel's air, and says so | `render.e2e.mjs`: air note checked word for word; the lit ocean is blue (blue channel 20+ above red) on both |
| Visual fixtures for Earth-, Mars- and Titan-like worlds (R2 row) | **Earth only in the browser**: the sample Sol has no Mars or Titan with surface tiles. The Mars and Titan air is checked in Node. Browser fixtures wait for grown or imported systems with those worlds (open). |

Screens: `R2-screens/true-colour-gpu-high.png`, `true-colour-canvas.png`.

## Fudges (listed in docs/fudges.md, Light, rendering and sound)
- **Air optics in the renderer, not the kernel.** Rayleigh coefficients are physics and belong in atoms. Asked the light and nebulae thread (19 Nov) to publish air optics per body; this file goes when they do.
- **Refractivity without dispersion.** One refractivity per gas near 550 nm, so the lambda^-4 law carries the colour; real blue scatters a few per cent more.
- **No aerosols or absorption.** No Mie haze, no ozone, no methane absorption, so Earth's limb lacks its haze and Titan is blue-grey rather than orange.
- **Single scattering.** Thick air (Venus, Titan) is too dark at depth; Hillaire's multiple-scattering LUT replaces it.
- **Palette hue with kernel albedo.** Ground colour is the survey palette's hue scaled to the kernel's albedo; real spectral reflectance per material comes with the light track.
- **Light direction is a look setting.** The star's light comes from the recipe's azimuth and elevation, not the star's real direction at the ruler's time, and the star's colour is white.
- **Canvas air is one column.** Plane-parallel air mass along the view and the sun, without a limb glow beyond the disk.

## Known limits
- Level-0 tiles only, so coastlines are blocky at the 25 km sample size (R2b).
- On WebGL2 (as in CI), gpu-high is the same code with more steps; true WebGPU was not available in this sandbox.
- The shell's limb glow beyond the disk is drawn but faint on the chart room's light page; a space background for photographic recipes is a later choice.

## Decisions needed from Alex
None blocking. One for later: should photographic recipes draw a black space background behind the globe (it reads better), or keep the chart room page?

## Follow-up (20 Nov): air optics from the kernel
The renderer's own Rayleigh table is gone. The Body screen asks the kernel (`read_air_optics`, light track, PR 24) for each world's air; `atmosphere.ts` only builds the request from the body's facts and reads the answer (`airRequest`, `airFromOptics`). `Session.readAirOptics` was added for it. `atmosphere.test.ts` now runs Earth, Mars and Titan through the kernel: Earth within 3% of Bucholtz, scale height 8.4 km; Mars about 11 km; Titan optically thick in blue. Gases the kernel's table lacks (NH3, SO2, CO, Ne and others) are left out and named in the badge, as before. The two renderer-side fudges (air optics in the renderer; one refractivity per gas) are removed from `docs/fudges.md`.
