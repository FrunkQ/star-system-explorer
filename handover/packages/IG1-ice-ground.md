# IG1: ice ground

Proven core, 25 Nov 2026. Asked for by the detail ladder thread (every icy world's tiles were rock, albedo a flat 0.3) and the renderer's MATCH1 photo matches.

## What it does

- **Ice share** (`sse.surface.ice_share_from_density`, Showman and Malhotra 1999): a small world's (below 0.1 Earth masses) share of ice from its mean density, rock and ice mixed by volume, the densities from the data sheet's surface rock and surface ice. A typed makeup wins.
- **Open ice lost** (`sse.surface.open_ice_loss`, Watson, Murray and Brown 1961; Fray and Schmitt 2009; Huebner et al. 1992): on an airless world open ice sublimes at its vapour pressure (Hertz-Knudsen, the sheet's vapour curve through the climate's `p_sat`), and each ballistic hop loses a share to photolysis (rate falling as 1/d²) or Jeans escape. Ground is ice where the loss over the world's age is less than the shell the ice would make. Under air (1e-4 bar or more) the vapour stays in the air.
- **Dark lag** (`sse.surface.ice_dust_lag`, Spencer 1987; Moore et al. 1999): open ice darkens with exposure age (the same age SW1 uses: surface age or the youngest crater's) towards mature mare soil, one fitted time (2.3 Gyr).
- **Atlas** (`Atlas::with_ice_ground`): an ice world's land, basin fill and worn debris are `ice`; an airless world's worn debris is `regolith`, not `sediment`. Bare airless ground and ice ground take their 550 nm reflectance from the sheet (`ice_ground::ground_optics`): basalt from immature mare soil, regolith from mature highland soil over SW1's factor, ice from water's condensed colour. SW1 weathering then darkens the silicate ground; ice is not weathered, it gathers the lag.
- **Records**: the survey has `ice_ground` (ice, ice_share, shell_km, lost_km, why); a resolved system's surface record carries it where the ground is ice.
- Test helper fix: `sol_tests::open_sol_with` gave every moon of a planet the address body:0, so only the last moon was kept. Each moon now has its own slot.

## Results (real Solar System, Orbit level)

| World | Ground | Mean tile albedo | Measured geometric |
|---|---|---|---|
| Moon | basalt and regolith (ice lost) | 0.13 | 0.12 |
| Mercury | basalt and regolith | 0.13 | 0.14 |
| Europa | ice | 0.32 | 0.68 |
| Ganymede | ice | 0.27 | 0.44 |
| Callisto | ice | 0.24 | 0.19 |
| Ceres | regolith (ice lost at 166 K) | 0.14 | 0.09 |

## Caveats (recorded, not fixed)

- Europa and Ganymede come out too dark: their exposure age is the whole surface age, because resurfacing from ice-shell activity (`shell_activity`) does not yet set an ice world's surface age.
- Titan has no measured air or temperature in base-sol, so it is treated as airless at 82 K (the fix is a base-sol row: 1.467 bar, 94 K, Fulchignoni et al. 2005).
- Io's sulphur and frosts are not counted (0.11 against 0.62).
- Ice reflectance is water's authored condensed colour; the data sheet needs a measured frost reflectance row.

## Golden

Re-blessed on purpose: the base-physics pack gains three blocks (atom count 150 to 153), so the pack hash lines move, and surface tiles change for icy and airless worlds.

## Next

Surface age of ice worlds from shell activity; Titan's air; then the crater and basin package (feathered basins with relief, apex-antapex lopsiding on locked worlds, oblique craters, young craters' own dates for rays).
