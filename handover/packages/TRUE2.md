# TRUE2 and TRUE3 part 1: plumes above the limb and hot things glowing, drawn in True

Track TRUE-surface (thread "Build plumes, frosts and hot glow"), third V3 audit items R10, R11 and R12.

## What
- `packages/renderer/src/plumes.ts`: the kernel's plume facts (TRUE1, `resolved_system` `surfaces[].plumes`) as grains in flight over the vents. The vents lie in the kernel's polar cap. Each grain flies on a ballistic arc to the height its launch speed gives, h = R v² / (v_esc² − v²), and escaping grains are drawn leaving. Brightness is the plume's strength times forward scattering (Henyey-Greenstein, g 0.7, a stand-in until the light thread's phase tables, K-e), so a back-lit plume shines and a front-lit one is faint. A note line says which pole, how high and how much.
- `packages/renderer/src/hot-glow.ts`: a world's own visible light by Planck, seen by the eye (CIE 1931), over the radiance of a white surface in its star's light (the star's radiance × (R★/d)²). True lights every world as if in its own daylight, so the glow is shown at that exposure. A 1500 K hot Jupiter at 0.05 AU is an ember (about 5×10⁻⁴ of its daylight). A 1300 K brown dwarf at 50 AU outshines its daylight. Nothing glows below the Draper point.
- Hooked in through the one door, `trueLook` (true-body.ts): new optional `TrueBody.heat` and `TrueBody.plumes`, and new optional `GpuItem.glow` (an added skin over the globe) and `GpuItem.plumes` (added points), each drawn in its own block in gpu.ts.
- Data path: `surfaceTiles` (ui-contract session.ts) also returns the world's surface `record`. SystemChart keeps each world's plume facts beside its surface and hands them to System3D (`plumes` prop). System3D gives `trueLook` each world's temperature and its star's temperature, radius and distance.

## Checks
`npm run ui:check`; `packages/renderer/test/plumes.test.ts` (facts read, grains over the cap and under the escape-limited height, back-lit bright and front-lit faint, trueLook turns grains to the pole, a brown dwarf glows and a cold moon does not) and `hot-glow.test.ts`.

## Not yet
- Live check (audit R3): Enceladus, Io and Triton are not in the default Solar System until the kernel's barycentre PR lands.
- Io's lava glow needs per-tile temperatures (VP-P1, through the detail ladder). The Lineup3D gallery does not pass `heat` or `plumes` yet: it needs the same surface record.
