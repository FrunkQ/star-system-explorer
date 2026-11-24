# Status: Stars on screen (glow, churn, flares)

Owner thread: "Build star glow and flares". Covers: SG (ordinary stars made dynamic on screen: the glow past the edge, a churning surface, lumpy giants, flares and mass ejections you can see; then star jets and shells, gap 8 of the third V3 audit). Alex's yes: 24 Nov 2026 16:41 ("yes to all", item 2 of the coordinator's list).

Only the owner thread edits this file. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

Files (agreed with "Build photoreal renderer", 24 Nov): `packages/renderer/src/star-glow.ts`, `star-churn.ts`, `star-events.ts`, `star-now.ts` and their test `star-now.test.ts` are this track's; so are `star.ts` and `live-star.ts` (handed over by the renderer thread, defaults kept so the Body screen's StarDisc looks the same). In `packages/views/system-3d/src/System3D.svelte` this track touches only the star's face block (`faceFor`) and its one glow sprite (`GpuGlow`); `gpu.ts` is the renderer's. Exotic stars, pulsars and black holes are "Build exotic stars and black holes"; `shownFlares`' three-flashes rule is shared with it.

Everything drawn comes from the kernel's `read_activity` (O10): spin, spots, the convection (pressure scale height, cell size, life), flares with their 10,000 K patch and area, and ejections with speed, mass, direction and cone. No kernel change and no golden change.

| Package | State | Log |
|---|---|---|
| SG1 the glow past the edge: the eye's glare (CIE general disability glare equation) summed over the drawn disc, at the worlds' daylight exposure, brightened by flares; an angle on the screen, so the same few degrees near or far | done | packages/SG1-3.md |
| SG2 a churning surface: each cell born, domed and fading over its own life on its own clock, drifting; a picture spanning many lives averages them (1/√N); the limb lifted by the cell under it by the kernel's scale height, so red supergiants are lumpy and the Sun is round | done | packages/SG1-3.md |
| SG3 flares and ejections you can see: the kernel's white-light patch (area, 10,000 K) where the ejection leaves; the ejection's front travelling at its speed, its Thomson-scattered light drawn as a coronagraph shows it; never more than three flashes a second, none under reduced motion | done | packages/SG1-3.md |
| SG4 the Body screen's star disc: the same churning cells (held at the star's present), lumpy limb and the kernel's flare patches as True; the `light` band typed as the sheet's `ViewBand` | done | packages/SG1-3.md (SG4 section) |
| SG4b the gallery's stars through the renderer's shared True look (`true-body.ts` `trueLook`, PR 442): face and glow from `drawStarNow` | next, when PR 442 is on main | |
| SG5 star jets and shells in True (third V3 audit, gap 8): young stars' jets and dying stars' shed shells from the light track's nebula kinds | queued | |
| SG6 prominences at the limb and filaments on the disc in hydrogen light (nebulae design 4A), once the kernel lists them | queued (needs the light track) | |

Phone: the face and the glow are CPU-drawn pictures (160 px glow, 16 to 256 px face) shown as sprites, so the WebGL2 path gets exactly the same star as WebGPU. The canvas tier keeps the chart look (True needs a GPU).
