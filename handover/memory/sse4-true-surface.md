---
name: sse4-true-surface
description: Track TRUE (thread "Build plumes, frosts and hot glow"): what is merged, what is open, and the agreed seams for surface colours (TRUE4 stains)
metadata:
  type: project
  modified: 2026-11-24T22:07:25.568Z
---
Track TRUE, third V3 audit items 4, 5, 6 and 10. Status file: handover/status/true-surface.md.

- TRUE1 plume facts (crates/sse-surface/src/plumes.rs, on resolved_system surfaces[].plumes) merged as PR 449 on 24 Nov.
- TRUE2 plumes drawn, TRUE3 part 1 hot glow, TRUE4 part 1 stains and TRUE6 lava tube openings merged in PR 492 (25 Nov 07:55, f00f882). Stains reach both the flat picture and the GPU face pictures (composeFaces(tiles, stains), kept on Composite.stains for withFiner), laid over withGround. Main moved fast: CI does not run on a conflicting PR, so merge main and push right after ui:check and ui:test, then run the long checks while CI runs.
- Earth at −18 °C is gone (288 K on the catalogued Sol, 24 Nov).
- Small bodies and comets (TRUE5, VP-P3) went to "Build asteroids and rings up close".

TRUE4 surface colours, as agreed on 24 Nov:
- Renderer: colour from data-sheet n,k through Hapke equivalent-slab grains plus intimate mixing (renderer sheet-optics.ts). The work in progress was uncommitted in worktree /home/user/sse-true2.
- Kernel: a new crates/sse-surface/src/stains.rs, written here and reviewed by proven core. It puts a world-level `stains[] {material_id, share, why}` on the surface record, as plumes do. All four stains are process strengths:
  1. ferric, from the survey's wet_myr;
  2. tholin, from CH4 in the climate exchange, falling back to early.outgas_ch4 (fudge row);
  3. sulphur and SO2;
  4. N2 and CH4 frost, from ice_kg_m2.
- Never add a field to Body: proven core's Body change is queued. The change goes through the golden lane via the coordinator.
- Data rows: the data sheet thread's PR 475 brings Mars dusts, hematite, limonite, basalt and water ice. This thread adds SO2 frost and the sulphur allotropes to scripts/optical_rows.py once 475 merges, and logs them as MP34. The data sheet thread keeps CO2 frost, the visible k of CH4 and N2 ice, and lunar regolith.
- Known gap: Mars's oceans = 0 in base-sol, so WaterHistory::steady gives no wet span, and ferric needs an oxidising-air term too.

Related: [[sse4-visual-polish]], [[sse4-exotic-build]].

- 25 Nov: PR 492 accidentally committed a worktree's node_modules symlink, because `.gitignore` had `node_modules/`, which matches only a directory. Removed in PR 541. Before `git add -A` in a worktree with a symlinked node_modules, check `git status`.
- Stains on Sol wait on proven core's PR 417 (SOL5 part: `orbit_atlas_with` and gases in the tile atlas), which is queued in the core golden slot after 463. Don't copy it. Probe: read level-0 tiles in a scratch sol_tests test and print `surfaces[].stains`. Before 417, only Earth stains (from water).
