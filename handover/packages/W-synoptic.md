# W-synoptic: a day's clouds instead of the season's average

Thread "Build shared climate and weather", 25 Nov 2026. It was asked for by the renderer thread's MATCH1 sheet (`sse4/handover-screens/MATCH1-screens/earth-pair.png` in project files). Beside the Blue Marble, Earth in True was banded like a gas giant, under an even veil.

## Why it was banded
The weather field gave every cell its latitude's average cover, about 0.75 with an optical depth near 3 at mid latitudes. The cyclones only added round blobs to bands that were already cloudy. An average is right for a climate, but a snapshot does not look like that: a patch of sky at one instant is mostly clear or mostly overcast.

## What changed
- **`sse-weather` `synoptic.rs` (new), `scatter`.** This runs between the zonal cell and the formations, for every world with a condensate that is not a giant.
  - Each layer's average cover becomes the day's cover, the way statistical cloud schemes do it (Sommeria and Deardorff 1977; Tompkins 2002). A cell is cloudy where a Gaussian water field passes a threshold, and the threshold is set so the expected cover equals the average: `t = sqrt(1 + s²) Φ⁻¹(1 − C)`, `cover = Φ((η − t)/s)`.
  - The field's largest eddies are half the Eady wavelength 3.9 N H / f at the storm track, the same length the cyclones use. Finer eddies follow Nastrom and Gage (1985): k⁻³, then k⁻⁵/³ below 500 km, down to 150 km.
  - The field drifts with each layer's wind and renews itself over a cyclone's life, using two slices cross-faded.
  - Each layer has its own field mixed with a shared deep one: 0.4 of the shared field for the low layer, 0.8 for the middle and high layers.
  - Rain is scaled by the day's cover over the average, so it falls where the cloud is.
- **Fronts.** Each storm-track cyclone adds, in field spreads:
  - a cold front trailing toward the equator and bending west, with the warm conveyor belt just ahead of it;
  - a warm front with its shield on the poleward side;
  - the comma head wrapped round the low;
  - the dry slot behind the cold front (Carlson 1980; Browning 1997).

  These are drawn in an azimuthal plane about the low, so they hold near the poles. They fade out beyond each front's ends. The cyclone branch in `formations::imprint` no longer adds a round blob of cover; it marks the thick deck as nimbostratus and puts rain under it.
- **Storm-track deck.** Poleward of the Hadley edge, 35% of the low cloud moves to a mid-level altostratus deck, with the total kept (ISCCP, Rossow and Schiffer 1999). Before this, the middle layer was empty at mid latitudes.
- **Trade cumulus.** Over any sea within the trades, the low cover is at least 0.25 times the sea fraction (Warren et al. 1988; Norris 1998), tapered over 10° at the trades' edge. The subtropics were cloudless before.
- **Whole-cloud optical depth.** Once the cover is the day's, a cloud that is there is a whole cloud, so its optical depth is its ISCCP class's middle:
  - cumulus and stratocumulus 10;
  - altostratus 9;
  - nimbostratus 30;
  - cirrus 1.5.

  Each is then scaled 0.5 to 1.5 by how far past saturation the cell is. The thin veil was much of the "washed out" look.
- **`rules.rs` zonal cell.** The rising belt's towers now grow in between 45% and 75% of the season's peak rain, instead of switching on at 60%. The switch drew a seam at the belt's edges.
- **Pack.** base-weather `weather_formations.synoptic` holds every number, with eight new references.
- **Example.** `cargo run -p sse-weather --example cover_map -- Earth 200.5 out.pgm` draws a day's cloud as reflected light, one pixel per degree.

## Tests (`synoptic::tests`)
- The day's cloud at 40° and 50° has clear gaps and overcast. Before this package it had neither: every cell was about 0.85.
- Over 40 days and 48 places, the day's cover averages back to the season's within 0.08.
- A faster spin gives finer weather: there are more clear-to-cloudy edges round 45° on the fast-spinning Earth.
- `inv_phi` round-trips, and the value-noise spread matches its constant.
- The marine-deck test now switches the day's pattern off (infinitely soft edges), because the deck is a matter of the average.

## Goldens
`tests/golden/weather.json` and `weather-storms.json` change: field tiles and reports now carry the day's cover, the storm-track deck, trade cumulus and whole-cloud optical depths. Contract lines that carry weather fields change for the same reason (see the PR).

## For the renderer
True asked for 24 cells per face edge. That is about 400 km on Earth, coarse for fronts 250 km wide. The renderer thread will ask for 48, and 64 at close-up, once this merges.

## Fudge rows
- the day's cloud;
- fronts;
- the storm-track deck and trade cumulus;
- in-cloud optical depth;
- towers by degree.
