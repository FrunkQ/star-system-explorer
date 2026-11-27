# SG7: one set of active regions for spots, flares and prominences

Thread "Draw binary mass transfer and flares" (27 Nov 2026). Alex asked whether prominences should cluster round sunspots. They were placed independently: spots by the renderer (`star.ts` `spotsFor`, its own random places), flares and ejections uniform within 30° of the equator, prominences uniform. Only their counts shared spot coverage. Coordinator approved one active-region list in the kernel as a routine addition (core golden slot, first green wins).

## Kernel

Four formula blocks in `packs/base-physics/formulas/activity.json`, each with a Rust copy in `crates/sse-atoms/src/activity.rs` held to it by `active_region_blocks_match_their_sums` (atom count 181 to 185):

- `sse.activity.differential_rotation`: surface rate over the equator's, Snodgrass and Ulrich 1990.
- `sse.activity.butterfly_latitude`: mean latitude of new groups through the cycle (Spörer's law), straight from 28° to 8°.
- `sse.activity.joy_tilt`: 32.1° sin|φ| (Stenflo and Kosovichev 2012).
- `sse.activity.spot_group_life`: peak area over 10 MSH a day (Gnevyshev-Waldmeier, Petrovay and van Driel-Gesztelyi 1997).

`crates/sse-activity/src/regions.rs`: regions born as a Poisson process in fixed bins (stream `activity.regions.v1`), thinned by the cycle, at a rate that keeps the mean area covered equal to the star's spot coverage. Each has birth, growth, life, peak area, latitude, longitude in the star's frame (turning once a spin from age zero), Joy's-law tilt, Hale's-law leading sign and polarity separation. `alive_at(t)` needs no window, so a flare or prominence finds the regions alive at its own moment and every window agrees.

- Flares: each is sited in a region alive at its peak, picked by spot area now (`Flare.site`); an ejection leaves from over that site, turned into the frame fixed in space. A flare in a spell with no region keeps the old belt draw and the why says how many.
- Prominences: active ones lie along their region's polarity-inversion line (spine across the pair's axis); quiet ones on a decayed region's channel, drifted poleward at the meridional flow for up to 10 turns, no further than the polar crown. Neither forms without its region (`Prominence.region`). Slingshots unchanged.
- `read_activity` with `prominences: {from_s, to_s}` now also answers `regions` over that span.

## Renderer

`packages/renderer/src/star-regions.ts`: each region drawn as its two spots (leader bigger, ahead and nearer the equator), area from the kernel's rise and decay times, carried at the kernel's differential rotation. `spotsOf`/`spotsAt` use the regions when the kernel gave them (the old scatter only for a star read without a span); `flaresAt` puts a flare at its site; prominence drift uses the kernel's rotation law. `turnAt` now turns from age zero (the kernel's frame), not from the reading's present. The Body screen's star says how many regions and their mean latitude.

## Golden

`tests/golden/contract.jsonl`: one line (read_activity flares gain `site`; ejection directions move to their regions). Area `stars` (regions, sited flares and prominences) and `tests/golden/activity.json` re-blessed. Core lane.

## Checks

- `cargo test -p sse-atoms active_region`; `cargo test -p sse-activity active_regions`: the Sun's regions number a handful to a few dozen at once, live 2 to 200 days, sit within 35° and follow Spörer's law (early mean above 20°, late below 15°), Joy's law holds, mean area covered is the coverage (absolute range); every sited flare's region is alive at its peak and its ejection leaves at the site's latitude; every active prominence's region is alive at its birth; overlapping windows and single moments agree.
- Seen red: with flare siting removed, `flares_and_prominences_sit_in_the_regions` fails; with `f.site` ignored in `flaresAt`, `star-regions.test.ts` fails.
- `node --test packages/renderer/test/star-regions.test.ts` (rise and decay, the pair, flares at their site) and the live-star, star-now and prominence tests.

## Not done

- Star as the close-up centre: lifting the camera's floor alone would show a 256 px face and a 160 px halo blown up; it needs the face and halo drawn at screen resolution near the star. Its own small PR.
- Eruption legs: the drawing already fades an erupting arch as 1/r²; nothing in the physics here says the legs thin faster, so they are left.
- Dark filaments on the disc in a hydrogen-alpha band; magnetogram band from `leading_sign`.
