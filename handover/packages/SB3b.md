# SB3b: belt members and comets on screen in system True

Render audit (26 Nov 2026, https://claude.ai/artifact/AGx6F4eKYSR4rYBRnv3BCu) build item 2, "draw what the kernel already knows". Thread "Draw comets, small bodies and nebulae", 26 Nov 2026.

## What was built
- `ui-contract/src/session.ts`: `smallBody(ask)` on `ViewSession` and `Session` (kernel `small_body`, read only).
- `views/system-3d/src/small-sample.ts` (new): `readSmallSample` resolves the 1st, 10th, 100th and 1000th largest member of each belt (`SAMPLE_RANKS`, fudge SB-sample) one request at a time, places each on its orbit now (Kepler's equation from the kernel's mean anomaly), and asks an icy member's activity again at that distance (the kernel's default is perihelion). A belt that refuses a rank stops there.
- `views/system-3d/src/bodies.ts`: `small` on `Body3D`, `fromSmallSample`. `System3D.svelte`: a member passes its figure to the shared `trueLook` door (so it draws in its own shape), is drawn smaller than a world in Readable, and an active one gets a coma glow and two tails of grains in the star's frame (`cometLooks`), with a note saying what is drawn and each comet's facts (`data-true-small`, `data-true-comets`).
- `views/system-chart/src/SystemChart.svelte`: reads the sample once per system on the GM seat after the worlds' surfaces (they share the kernel) and adds the members to the 3D bodies in True only.
- `renderer/src/comet.ts` (new): `cometTails` (grains spaced as 1/s along each tail, a steady flow; each tail at the kernel's length, straight away from the star), `comaPicture` (light as 1/ρ, Haser 1957), `ionColour` (CO+ blue or H2O+ orange by the dominant ice), `showing` (brightness from production, fudge COMET-glow), `cometNote`.
- Fudge register rows SB-sample, COMET-glow, COMET-coma, COMET-tails; citations for Haser 1957 and Brandt and Chapman 2004 (unverified, Consensus out of allowance until 1 Dec).
- Status rows corrected: SB3 said True drew figures but no view passed one; SG5a said drawing done but nothing calls it; SG5b is blocked on the kernel (below).

No kernel, golden or `gpu.ts` change.

## Drawing paths covered
System True only (`gpu.ts` path through `System3D.svelte`). Not the Body globe (`gpu-globe.ts`): belt members are not scene subjects, so they have no Body page. Not Land (`gpu-surface.ts`). The size comparison and class gallery share `trueLook`, so they would draw a figure if handed one, but nothing hands them one yet.

## Evidence
- `packages/renderer/test/comet.test.ts` (6 pass): quiet bodies draw nothing; each tail reaches the kernel's length and no further, all on the side away from the star; grains thin as 1/s (half the decades hold half the grains, within 5%: an even spread would put under 1% there); brightness on its log scale; CO+ blue and H2O+ orange; the coma falls off and is dark at its edge.
- `tests/ui/system3d-small.test.ts` (3 pass): perihelion at a(1 − e), aphelion at a(1 + e), faster past perihelion; ranks sampled in order, an icy member asked again at its distance now, a refused rank ends that belt; stops when the view no longer wants it.
- In the browser (Chromium, WebGL2, built shell): seed "first light" shows four members of its debris belt; flying to the 10th largest shows a dark oblate Maclaurin figure, the 1000th largest of seed "comet" a lumpy irregular one. Seed "comet" draws four active members with blue CO+ ion tails pointing away from the star. Screenshots in project files `sse4/handover-screens/SB3b-screens/`.

## Seen on the way (kernel, not changed here)
- Ion tails come out very long: 55 to 81 AU for CO2-driven members at 6 to 8 AU (seed "comet"); real ion tails reach a few AU. The drawing follows the kernel; worth a look by the small-bodies track (`sse-small` comet.rs, ion tail length).
- Seed "cold harbour": S and Q type members 0.1 to 0.6 AU from the star are "active" at up to 4e7 kg/s of dust, more than any comet seen. A rocky body with a trace of ice so close in may lose that ice early in reality; worth a look with the activity rule.
- A member chosen in True leaves the card at "Nothing selected": members are not scene subjects, so the card has nothing to show. A member card is a chart-room or selector job.

## Not done
- Nebula shells and jets in True (SG5b): read_nebulae only answers for giants and young stars it is handed with their start dates and accretion rates. No op gives a view those from a star's life line; the light track has this as its next step after O3b.
- Aurorae in system True: needs the aurora shells added to `gpu.ts`'s draw loop, which the shadows work is editing now; left for after it lands (or for "one face for every view", audit item 5).
- Members do not move on their orbits (the worlds do not either in system True).
