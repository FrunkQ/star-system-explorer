# Status: Galaxy charts and histories

Owner thread: "Build galaxy charts and histories" (split from "Build galaxy charts and travel screens" on 24 Nov 2026; travel keeps V8, voyages, the flight log and the travel panels). Covers: U6, U8, V9, GH8, V3 gap 39.

Files: `packages/views/galaxy-chart`, `packages/views/galaxy-3d`, `packages/views/foam-map`, `packages/views/cluster-map`, `packages/views/story-finder`, `packages/ui-contract/src/galaxy.ts`, `tests/ui/galaxy*.e2e.mjs`, `tests/ui/foam*`, `tests/ui/cluster*`, `tests/ui/stories*`. The True picture (GH9, `GalaxyScars`, renderer galaxy files) is the renderer thread's; the 3D and True web cube (`foam-map/src/Web3D.svelte`, renderer `web-cube.ts`) is "Build cosmic web view"'s.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

| Package | State | Log |
|---|---|---|
| U6 galaxy and patch charts (part 1: Galaxy under the Explore tab, from the `galaxy_chart` op) | parts 1, 2a (inked history preview) and 2c (what other galaxies brought, GH3) done; 24 Nov: the stellar halo row is `data-stellar-halo` (it clashed with the halo picker's `data-halo`); next: part 2b, patch survey, find a home, save inked history | packages/U6.md |
| V3 gap 39: system hover summary (GM only: counts of stars, planets, moons, minor bodies, ships, any life, anything remarkable; the selector of gap 37 reads the same summary) | part 1 done 24 Nov: pure reader `ui-contract/src/system-summary.ts` (`systemSummary`, `summaryWords`; reads scene subjects or picker chips; life only from life/biosphere tags or a Life chip, never guessed), test `system-summary.test.ts`; the navigation thread puts it on the picker's system rows; next: the hover on the Your systems chart (gap 17) | |
| GH8 leading-arm mark in the chart's history (design SSE4-galaxy-histories-design.md 9.4; rare, about 1 in 100, a cited editable number) | not started; waits on the kernel's GH3 mark and arm sign fix ("Build galaxy arm fix and atmosphere advice") | |
| V9 map views (Cosmic web foam under Explore from `web_slice` with time scrub and Play; cluster map "Galaxies nearby" on `halo_cell`) | parts 1 and 2 done | packages/V9.md |
| U8 story finder (Stories under Explore, on the `sift` op; starter sifters as data, hits with evidence, the GM's own sifter, a storyteller brief) | part 1 done; Minerva and rarity next | packages/U8.md |
