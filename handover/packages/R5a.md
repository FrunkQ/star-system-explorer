# R5a: stars on the Body screen, drawn from their physics

Status: done (20 Nov 2026), renderer track, PR into `main`. First package of R5 (exotic objects through light lenses), proposed 19 Nov after Alex asked for black hole discs, gamma views and an improved magnetic field overlay.

## What was built (three lines)
1. `renderer/src/star.ts`:
   - Grey-atmosphere limb darkening (Eddington).
   - Spots covering the kernel's `spot_coverage`, in belts that follow Sporer's law through the cycle (high-latitude spots on stars with no clean cycle). Each spot is drawn by its own blackbody per channel.
   - Coronal scale height kT/(μ m_H g).
   - An X-ray view: the optically thin corona's emission measure along each line of sight, with limb brightening and active regions, on a labelled mapped-colour scale.
2. `views/body/src/StarDisc.svelte`: selecting a star on the Body screen now draws it, where it used to say "the Body view shows worlds". It shows:
   - temperature, mass and radius from `resolved_system`
   - spin, regime, spots, cycle, X-ray power and the coming year's flares from the kernel's `read_activity` (O10)
   - a Visible / X-ray switch, turning with the arrow keys, and words for screen readers
3. `Session.readActivity` (one read-only method).

## Results
| Check | Result |
|---|---|
| Limb darkening, spots, scale height (`star.test.ts`) | 1 at the centre and 0.4 at the limb; coverage within 35% of the kernel's; mean spot latitude above 22° early in the cycle and under 14° late; same spots each time; Sun's corona H ≈ 0.11 R☉ (75 Mm) at 1.5 MK |
| Pixels | Limb darker than the centre; a spot darker than the photosphere; 9600 K bluish white; in X-rays the rim outshines the disc centre |
| Browser (`render.e2e.mjs`) | The Sun: "moderate, turning every 26.1 days (age); 7 spots covering 0.28%, at about 23°, 23% through the cycle"; next flares above 10²⁴ J listed; the X-ray view is mapped violet; the state hash is unchanged |

Screens: `R5-screens/sun-visible.png`, `sun-xray.png`.

## Fudges (in docs/fudges.md)
Spot maps drawn rather than modelled; one spot contrast; corona at 1.5 MK; X-ray active regions as brightenings; grey limb darkening in every band.

## Next (R5b onwards)
- Accretion discs and jets through the light lenses.
- Gamma and radio views.
- Bands taken from `read_light` instead of the two fixed views.
- The star view moving into the recipe runtime with a GPU tier.
- The magnetic field overlay (R2c) once the physics track publishes field geometry.
