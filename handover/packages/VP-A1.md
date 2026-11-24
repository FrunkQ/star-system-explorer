# VP-A1: giants carried by their winds (differential rotation and turning storms)

Design: `handover/design/SSE4-visual-polish-design.md` section 5.4 (accepted by Alex 24 Nov 2026, "yes to all").
Owner: "Build giant bands, clouds and hazes" (status `handover/status/true-air.md`).

## What it does

A giant's True face, as `readTrueFace` makes it, now comes with its flow: the eastward and northward wind of the
highest showing deck at each pixel, blended from the weather op's cells exactly as the colours are, each row's mean
eastward wind, and the storms alive that day (`want: "systems"`, where each stood). `flowFace(face, flow, { radius_m,
dt_s })` gives the face `dt_s` seconds later:

- each row slides east by u Δt / (a cos φ), so the equatorial jet laps the temperate bands (Jupiter's equator
  laps System III by about 7° a day at 100 m/s);
- each storm moves as one piece at its centre's speed (the weather block's own drift rule) inside a Gaussian of its
  radius, so the bands do not shear it apart;
- each storm turns at the angular speed of the cells' winds round its centre, drawn in cycles of a quarter turn at its
  edge with two phases blended, so the faster core never winds into a spiral.

Grade (design section 3): fact. Every speed is the kernel's; nothing here makes up a wind. No kernel ask was needed:
the cells already carry each deck's wind, including each storm's own vortex (K-a not needed).

`GpuItem.surface.version` (optional): a new version uploads the new pixels into the same texture. The first upload is
now a copy, so the caller's face is never written through.

## Where it shows

The size comparison's turning globes (Sizes, 3D look True). `Lineup3D` now reads each of the system's worlds through
`readTrueFace`, so a giant there is its cloud decks (it had no face before) and a rocky world its ground with today's
clouds. The renderer's one door `trueLook` (`true-body.ts`) takes an optional `flow: { flow, dt_s }` and hands the
GPU the carried face with `version: dt_s`; `bodySeconds` (`lineup-3d.ts`) turns the seconds watched into the body's
own seconds at the pace `spinAt` turns it. The system chart has no running clock, so it shows the face as read.

## Checks

- `packages/renderer/test/true-air.test.ts`: a fast equator moves a marked meridian by u Δt / (a cos φ) within a
  pixel while a still band stays; a storm's dark half goes round anticlockwise with its anticlockwise winds; a still
  storm in a 60 m/s wind is carried 10° east pixel for pixel; no time, no change.
- `trueLook` with a flow hands over a new picture and its version; `bodySeconds` gives one body day per turn watched
  at "same" pace and the quickest body's day at "true" pace.
- `npm run ui:check` clean.

## Fudges

`docs/fudges.md`, "Giants carried by their winds (VP-A1)".

## Integration points

- Renderer thread: `TrueBody.flow` (optional) in `true-body.ts`, `bodySeconds` in `lineup-3d.ts`, `GpuItem.surface.version`.
- Climate and weather threads, K-a (shape agreed 24 Nov): the giant output gains `jets` with `lat_deg` and
  `jet_speed_m_s`, `jet_shear_m_s`, `rhines_scale_m`, `deformation_radius_m`. Once it lands, the row speeds can be
  read from `jet_speed_m_s` in place of the cells' mean wind (they should agree; a test can hold them to it), and
  VP-A2's curls take their size and strength from the other three.
