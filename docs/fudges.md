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

## Light, rendering and sound

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|
| `packages/renderer/src/atmosphere.ts` (R2a) | Air optics (Rayleigh per gas, scale height) computed in the renderer | The kernel op did not exist yet | Kernel `read_air_optics` (light track, PR 24) | Stand-in until PR 24 merges; then the renderer reads the op |
| `atmosphere.ts` gas table | One refractivity per gas near 550 nm, no dispersion | Small table, colour carried by the lambda^-4 law | Per-wavelength refractivity (Peck and Reeder, Bucholtz tables) | To review with the kernel op |
| `gpu-air.ts`, `canvas.ts` | No aerosols (Mie haze) and no absorption (ozone, methane) | Not built yet | Mie scattering from the weather op's aerosols; absorption bands from the light track | Stand-in until R2b |
| `gpu-air.ts` | Single scattering only | Exact for thin air, no lookup tables to build | Hillaire (2020) multiple-scattering LUT; thick air (Venus, Titan) is too dark without it | Stand-in |
| `gpu-globe.ts` physical lighting | Ground hue from the survey palette, scaled to the kernel's albedo | No spectral reflectance per material yet | Spectral reflectance per material (light track) | Stand-in |
| `passes.ts` relief pass | The star's direction is a look setting (azimuth, elevation) and its light is white | The view has no ephemeris position at the ruler's date yet | Star direction from the orbit at the campaign date; star colour from its spectrum | Stand-in |
| `canvas.ts` air | One-column, plane-parallel air along view and sun; no limb glow beyond the disk | Canvas tier is the weak-device path | The GPU ray-march (already on GPU tiers) | Permanent for the canvas tier (weak devices do less) |

## Interface and player views

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Candidates to move into flows

Logic written in Rust that could be an editable, cited flow of small blocks (Alex, 19 Nov: "try and do everything in
the physics chain blocks... other than Big Blocks. And have a UI element that makes sense to visualise").

| Where | What it computes | Why it is not a flow yet | Blocks it would need | View that would show it |
|---|---|---|---|---|
