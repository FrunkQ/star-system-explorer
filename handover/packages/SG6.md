# SG6 part 1: prominences at the limb

Thread "Draw binary mass transfer and flares" (26 Nov 2026), from Alex's picture of a star with loops leaping off its limb ("real calc'd from magnetism... we were having it at one point"). What was there: flares (white patches) and mass ejection fronts in System True (SG3). Prominences were designed (nebulae and light design 4A, 19 Nov) and queued as SG6, never built.

## Kernel

Four new formula blocks in `packs/base-physics/formulas/activity.json`, each with a Rust copy in `crates/sse-atoms/src/activity.rs` held to it by `prominence_blocks_match_their_sums`:

- `sse.activity.surface_field`: mean surface field from the Rossby number, Reiners et al. 2022's slow and saturated branches. The Sun: 95 G; a saturated star: about 2.8 kG.
- `sse.activity.corona_temperature`: Johnstone and Güdel 2015, 0.11 F_X^0.26 MK. The Sun: 1.7 MK.
- `sse.activity.corona_base_density`: the corona as one scale height glowing optically thin, L_X = n² Λ 4πR² H. The Sun: 1.9e14 m⁻³.
- `sse.activity.closed_corona_radius`: the Sun's source surface (2.5 R) scaled as the cube root of the field.

The co-rotation radius reuses `sse.orbit.semi_major_axis_from_period` (Kepler with the spin period); its Rust copy is `corotation_radius`. Atom count 176 to 180.

`crates/sse-activity/src/prominence.rs`: `corona()` and `prominences()`. Births are a Poisson process thinned by the cycle, in fixed bins per kind with their own draw streams (`activity.prominences.<kind>.v1`), so overlapping windows agree. The number at once follows spot coverage (about 12 for the Sun). Heights are a share of the corona's pressure scale height (`sse.atmosphere.scale_height` at μ 0.6). The gas sits in pressure balance (n_corona × T_corona / 8000 K), which gives its mass (Sun: 4e16 m⁻³, 1e11 to 1e12 kg). Brightness: hydrogen-alpha lit by the diluted disc, about 1e-4 of the disc. A fast spinner whose co-rotation radius lies inside its closed field also holds slingshot clouds there (AB Dor: 2.7 R inside 7.8 R), each flung off at the end of its life at the speed it is whirled round at.

Op: `read_activity` takes optional `prominences: {from_s, to_s}`; each star's answer then carries `prominences` (corona, list, mean at once, why, stand-ins, references). Without it, answers are unchanged.

Every stand-in is in `docs/fudges.md` (Light, section 3); citations in `docs/citations/stars.md`, all unverified until the paper search returns (1 Dec 2026).

## Renderer

`packages/renderer/src/star-prominences.ts`: `prominencesAt` places each prominence on the turning star at star time t (carried at its latitude's rate, grown in and faded out over a tenth of its life, an eruption rising at its speed and thinning as 1/r²). `addProminences` draws arches (three threads along a parabola over the spine) and slingshot balls into the halo picture, only off the disc, coloured by the kernel's own hydrogen lines, on a hydrogen-alpha filter log scale (`PROMINENCE_SCALE`). `narrateProminences` says how many stand at the limb, the field holding them, and that the eye sees them only in an eclipse. `drawStarNow` calls them, so System True's star shows them on its own star clock (`ask.t`). `readLiveStar` asks for 90 days of the star's time (about six minutes on the six-hours-a-second clock).

## Golden

Formula blocks are compiled in, so the manifest and stack hashes move in the core contract golden (core lane). No op output changes unless `prominences` is asked for.

## Checks

- `cargo test -p sse-atoms` (block against sum; absolute Sun ranges), `cargo test -p sse-activity prominences` (Sun ranges absolute: height 3e6 to 1.5e8 m, density 5e15 to 1e18 m⁻³; AB Dor slingshots at its co-rotation radius; overlapping windows agree; a 9500 K star has none).
- `node --test packages/renderer/test/star-prominences.test.ts`: drawn at the limb, not on the disc; colour from its lines; growth, fade and eruption; reduced motion holds; the living star's halo and words.
- Seen red: with the off-disc test removed from `addProminences`, the "facing us it does not" check fails.

## Next

- Dark filaments on the disc and limb arches together in a hydrogen-alpha band in the band switch.
- The Body screen's star (StarDisc) and its "Hide the disc" corona.
- Hydrogen-alpha light transfer through the slab as a block (fudge register candidate).
