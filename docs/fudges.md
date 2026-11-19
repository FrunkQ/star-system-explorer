# Fudge register

Alex, 19 Nov 2026: "the physics is WAY more interesting than faking it. But understand a browser has limited
capability. Fall on physics rather than simplification where practical. If we decide to "fudge" it is a known and
documented fudge with a damn fine reason to exist."

A fudge is any deliberate simplification, stand-in, tuned constant or placeholder that is not the published
physics. Each one is listed here, in the PR that adds it, by the thread that adds it.

Columns: **where** (file and item), **what it does instead of the physics**, **why** (the damn fine reason:
browser cost, missing data, not built yet, tuned to match an observation), **real physics that would replace it**,
**status** (stand-in until package X, permanent with reason, to review).

Add entries under your area. Keep one row per fudge.

## Stars

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Orbits and formation

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Interiors and heat

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Surfaces

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Climate, weather and life

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Galaxy and cosmology

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|
| `sse-kernel/src/cosmos.rs` `galaxy_chart` arm crests | Arms drawn as crests of one fixed log-spiral density wave (envelope pitch and pattern speed), rotated to the time shown | The envelope's arm term is itself that pattern; a chart needs lines | Transient, ragged arms from a stellar-dynamics model, broken up after mergers (G6 events) | stand-in until arms are evolved |
| `sse-kernel/src/cosmos.rs` `web()` | Cosmic web octaves capped at 4 for `galaxy_chart` and `web_slice` | Browser cost: a 96 x 96 slice in about a second | All pack octaves (finer filaments below a few Mpc) on capable devices | permanent cap for weak devices; to lift by tier |
| `sse-kernel/src/cosmos.rs` `galaxy_chart` halo | The median halo for a named mass, not the campaign's own galaxy | G7 (Milky Way pack, campaign galaxy) not built | The campaign's galaxy placed by G7 | stand-in until G7 |

## Light, rendering and sound

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Interface and player views

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|
| `ui-contract/src/route.ts` `navFromScene` | Orbits taken as flat (inclination 0) and moons left out when the chart scene feeds NAV | The chart scene carries no inclinations or moon elements | The adopted system's SystemSpec elements (the `system` source of `plan_route`) | stand-in until scenes carry elements |
| `sse-nav/src/sky.rs` `window` albedo | Every body reflects at geometric albedo 0.3 (pack `nav.window.default_albedo`) | The chart has no albedos yet | Each body's albedo from its surface and clouds (S7 cover, W weather) | stand-in until bodies carry albedo |
| `sse-nav/src/sky.rs` `window` star brightness | The star's magnitude scales with bolometric luminosity, not its visual band | Needs a spectrum to integrate | V-band flux from the star's spectrum (O1/O12 `read_spectrum`) | stand-in until the light track's spectra are wired in |
| `packs/base-nav` `nav.window.sensors` | Three sensors with rough limiting magnitudes and resolutions | Sensors by band (O track) not built | Sensor arrays by band with apertures, exposures and noise | stand-in until O-track sensors |
| `sse-kernel` `sift` rarity | Hits carry no rarity (no resolve clouds passed) | Not wired yet | Rarity from the parent's resolve cloud (`criterion_probability`) | stand-in |

## Candidates to move into flows

Logic written in Rust that could be an editable, cited flow of small blocks (Alex, 19 Nov: "try and do everything in
the physics chain blocks... other than Big Blocks. And have a UI element that makes sense to visualise").

| Where | What it computes | Why it is not a flow yet | Blocks it would need | View that would show it |
|---|---|---|---|---|
