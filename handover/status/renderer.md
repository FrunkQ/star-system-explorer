# Status: Photoreal renderer

Owner thread: "Build photoreal renderer". Covers: R, the drawing side of Y5 and MF14a.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`. Stage order: `handover/design/SSE4-coming-together.md`.

| Package | State | Log |
|---|---|---|
| R1 pass library, render recipes, degrade order, governor, quality badge (the Body globe runs on recipes: GPU, canvas, words) | done (renderer track) | packages/R1.md |
| V3 gap item 14 part 1: ship hulls and drive plumes from the ship record's physics (tanks, shield, thrust, jet power, words, Chart paths, lathe outline) | done | packages/SHIP14.md |
| V3 gap item 14 stage 2: ships designed from their parts (base-ships pack rules and styles, side and top views with call-outs, remix, hand overrides, ShipPlans view) | done | packages/SHIP14.md |
| V3 gap item 14 stage 3: the 3D model from the same layout (volumes and radiator area checked), panel textures and looks per style, glowing hot radiators, Chart/3D through the shared ViewControls in ShipPlans | done; stage 4 (abilities from the kernel's ship_at_world: heat shields, wings, legs, scoops) next | packages/SHIP14.md |

## Done so far and what is next

R1, R2a (air from kernel facts, physical lighting, True colour), R3 (still tier, "Take photograph"), R4 part 1 (checked recipe edits, hand editor) and R5a (stars: photosphere, spots, X-ray corona, granules from O10 cells), R6 (time-lapse layers, `packages/R6-timelapse.md`) and R2b part 1 (today's clouds, haze and storms from the weather op, `packages/R2b-weather.md`), Y5 parts 1 and 2 (the stars beyond, and behind the Window's lights, `packages/Y5.md`) and MF14a part 1 (the Mag layer's shared tracer, Chart cut and 3D lines, `packages/MF14a.md`) done; air from the kernel's read_air_optics; R2b part 2 (cloud shell at height on the GPU tiers), MF14a part 2 (3D tubes and True band emissions), R5 light-lens views, R4 part 2 with Orac (A2)
