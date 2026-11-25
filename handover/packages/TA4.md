# TA4: a giant's deep air and the gas over its decks (methane), and warmer belts

Track TA (`handover/status/true-air.md`). Renderer and ui-contract only; no kernel, pack or golden change.

Why: the renderer thread's MATCH1 sheets (25 Nov, project files `sse4/handover-screens/MATCH1-screens/`) showed Uranus and Neptune white-grey where they are cyan and blue, Jupiter pastel pink and Saturn pinkish where they are tan and butterscotch.

## What it does
- `readGiantGas` (`packages/renderer/src/true-air.ts`): for a `cloud-deck` world, asks the kernel for Rayleigh cross-sections (`read_air_optics` at `CHANNEL_NM`) and, in one `light_through_gas` call, its absorbing gases per molecule of air (mole fraction over one metre at one molecule per m³) and the gas over the decks (`gasAbove`, from the 1 bar reference level up, lit by the system's star).
- `deepAirOf`: per wavelength the single-scattering albedo is Rayleigh over Rayleigh plus absorption; reflectance of a semi-infinite layer, two-stream isotropic, R = (1 - sqrt(1 - w)) / (1 + sqrt(1 - w)) (Bohren 1987); each channel the mean over its wavelengths. This is the colour below the decks; the gas tint goes over the decks.
- `albedoOfColour`: an aerosol's colour read as the look of a thick layer (the same reflectance inverted) instead of as an albedo directly, so a red-brown stand-in absorbs blue strongly and green moderately: belts go tan, not pink.
- `CHROMOPHORE_VISIBLE`: the chromophore's 1 µm optical depth carried to 550 nm by an Ångström exponent of 1.5 (stand-in, fudge register).
- `Session.lightThroughGas` (ui-contract): the kernel's `light_through_gas`, read only.

## Numbers (Sol, node kernel)
- Deep air, linear RGB: Jupiter 0.44, 0.76, 0.92; Uranus 0.20, 0.54, 0.80; Neptune 0.24, 0.58, 0.83.
- Gas over the decks: Uranus 0.74, 0.93, 1.00; Jupiter 0.97, 1.00, 1.00.
- Face means (sRGB): Uranus 167, 207, 235; Neptune 177, 213, 237; Jupiter 188, 164, 133; Saturn 181, 160, 130.

## A trap found on the way
A local `scripts/build-wasm.sh` can ship a stale data sheet: a worktree sharing `target/` left cargo's fingerprint for `sse-reference`'s `include_str!` pointing at the worktree's old `packs/base-reference/pack.json`, so the wasm kept methane rows 10⁴ too strong (fixed on main in PR 406). Never symlink `target/` into a second worktree; `touch crates/sse-reference/src/lib.rs` forces a fresh read.

## Still to do
- The chromophore's optics on the data sheet, so TA2's particle block sizes and colours it (drop the stand-ins).
- Decks at their own pressures with the gas between them (VP-A4 part 2).
- The Great Red Spot and storms in the system view's static face.
