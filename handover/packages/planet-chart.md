# Planet chart: moons in their planet's frame

Status: done (19 Nov 2026), branch `claude/project-thread-adgixl`, in the V4 PR into `main`.
Asked for by the coordinator on 19 Nov ("a planet-level view so moons have a home"); not a numbered package in the build plan. It applies `handover/rendering-rules.md` to the 2D chart.

## What was built
- **Frames** (`ui-contract/src/scene.ts`): `childrenOf(scene, centre)` lists the bodies orbiting a centre, nearest first; `systemOrder` walks star, then each planet followed by its moons; `frameFor(scene, address)` names the frame a body is drawn in (null for the star's, the planet for a moon); `KM_PER_AU` (IAU 2012 exact).
- **System chart** (`views/system-chart`): shows one frame at a time. The star's frame draws only the star's own children; a planet with moons carries small moon ticks (`data-moons` gives the count). Choosing that planet shows "Moons of <planet> ›" (double-click also opens it); its frame puts the planet at the centre, drawn larger, with its moons about it on the same log-radial grammar, distances in kilometres with 100,000 km and 1,000,000 km rings, no zones, belts or survey hatch. "‹ <system>" or Escape goes back with the planet still chosen. Choosing a moon anywhere (body cycling, another view) opens its planet's frame; choosing a planet returns to the star's. Dragging a moon moves its orbit as an intent, rounded to three figures in km and held in AU, and capped at twice the frame's outer edge.
- **Body card** (`views/body-card/src/parse.ts`): a distance held in AU also accepts kilometres ("500,000 km", commas or spaces grouping thousands).
- **Fixture** (`shell/src/documents/kestrel.scene.json`): Tarn a (Tarn's moon, same address as in the player facts, `body:3/body:0`) and Vault's three icy moons Hasp, Rivet, Latch (`body:4/body:0..2`). Moons have no derivation rule yet, so no derived facts.
- **Shell**: the body cycle keys follow `systemOrder` (each planet, then its moons), not raw distance.

## Acceptance
| Criterion | Where it is proved |
|---|---|
| Moons are never drawn in the star's frame | `ui-contract/test/scene.test.ts`; `tests/ui/shell.e2e.mjs` "Planet frame": no Hasp on the system chart, Vault reports three moons |
| A planet's frame shows the planet at the centre and its moons about it | shell e2e: the chart is labelled "Chart of Vault and its moons", bodies are Vault, Hasp, Rivet, Latch, legend says kilometres |
| Moon orbits edit like planet orbits, in km | shell e2e: Hasp shows 412,000 km, a drag outward gives a larger km value (477,000 km in Chromium), typing "500,000 km" sets it |
| Back to the system keeps the choice | shell e2e: "‹ Kestrel" returns to the system chart with Vault's card open |

Results: 76 UI node tests pass; svelte-check clean; `scripts/check.sh` passes; shell e2e passes in Chromium. Screens at 1280 and 390 px checked by eye.

## Not done, and why
- Moons have no derived facts (gravity, equilibrium temperature): `derivations.base.json` has no moon rule and a moon's star distance is its planet's, which the interim derivation does not yet compose. Comes with the kernel's own scene (K9) or a moon rule that reads the parent's orbit.
- Zooming does not yet hand over to a child frame automatically; opening is a button or double-click. The 3D views (V6) will do the handover by zoom as the rendering rules say.
- Player chart: the players' Chart draws the scope's top-level bodies; a player planet frame follows the same helpers when V10 or V6 touches it.
