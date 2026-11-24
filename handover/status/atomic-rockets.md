# Status: Atomic Rockets (track RK)

Owner thread: "Build ship flow and power views" (from 24 Nov 2026). Covers: the RK packages handed on by the chart room
and renderer threads: RK13 (Flow view and Power card), RK12 (range rings as a HUD overlay), RK10 (remaining band
strings), RK3 (ship sizing formulas as named Equations). Other RK packages stay rows in their owners' files: RK9 in
`galaxy-and-travel.md`, RK6 in the Paper spacecraft pack's thread.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are
in `handover/packages/`. Plan: `handover/design/SSE4-atomic-rockets-plan.md` (accepted in full, questions 1 to 25).

| Package | State | Log |
|---|---|---|
| RK13: the general Flow view (`ui-contract/flow.ts`, `views/flow` Flow) and the Power card on a ship's card (three columns, "Nowhere" dashed band on the GM seat only, five readings, live starlight and load switches) | done 24 Nov; next: `reject_k` on the kernel's heat rows (golden lane); later users of Flow: a world's energy budget (weather), an element's journey (matter) | packages/RK13-flow-and-power.md |
| RK12 part 2: range rings as a HUD overlay: `ui-contract/range-rings.ts` (`rangeRings`, `ringPoints`, layer `range`), session `effectAtRange`, `system-chart/RangeRings.svelte`, `system-3d/range-rings.ts` | done 24 Nov; next: the chart room mounts them and adds `range` to `HUD_OVERLAY_LAYERS`; a pack's weapons feed them | packages/RK12-rings.md |
| RK10 part 2: band strings in ui-contract `light.ts` (BANDS, the band-to-view map) read `SHEET_BANDS`; `view-controls.ts` already did; renderer `GALAXY_BANDS` and `bandOf` too | done 24 Nov; left: star.ts and live-star.ts types (star glow thread's files) | packages/RK10-ui-strings.md |
| RK3 (sizing share): ship sizing as named Equations, family `sse.ship` in `packs/base-physics/formulas/ship.json` (jet power, stowed volume, crew volume, waste heat, radiator area) | done 24 Nov, golden lane; next: a kernel op sizing a ship through them for the renderer | packages/RK3-ship-equations.md |
