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
| RK12: range rings (kernel `effect_at_range` done) as a HUD overlay: `ui-contract/range-rings.ts`, one component each in system-chart and system-3d; the chart room thread mounts them | next | |
| RK10: band strings in ui-contract `light.ts`, `view-controls.ts` (and the renderer's) read `SHEET_BANDS` | queued | |
| RK3: ship sizing formulas as named Equations in `packs/base-ships` (may change goldens: golden lane) | queued | |
