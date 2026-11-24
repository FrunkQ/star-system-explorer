---
name: sse4-night-sky-track-y
description: Night sky design (track Y) accepted 21 Nov 2026; which Y packages the kernel thread owns and the G5-not-wired finding
metadata:
  type: project
  modified: 2026-11-21T16:24:00.715Z
---
Alex accepted the night sky design on 21 Nov 2026 (SSE4-night-sky-design.md; stages in section 11, rulings in section 12, count tree in section 4).

The kernel resolve and rules thread owns these packages:
- Y0: the cloud shape field. It shipped in PR 73 as disc or sphere, then was renamed so the isotropic case reads "shell".
- Y7a: promote stores the full description; this lands before Y1.
- Y1: the count tree. Re-blessing the G5 golden is allowed by ruling.
- Y2: the `sky_from_here` op.
- Y4: galaxies in the sky.
- Y7: the catalogue intent.
- Y8: catalogue pins. This replaces G7's catalogue part.
- Y9: sibling clusters.

These are fitted between the E packages in section 11 order.

Finding (21 Nov): the G5 star draw in `sse-cosmos` stars.rs (`cell`, `promote`) is not wired into the kernel at all. No op reads star cells and no save holds promoted stars. Y7a therefore adds the kernel ops and record storage, and the save migration has nothing to migrate yet.

**Why:** Gaia-derived data must live only in the optional real-sky pack, never in the app (ruling Q3).

**How to apply:** follow section 11 order. Keep every Gaia-derived dataset out of the app's packs.

See [[sse4-build-conventions]].

Also queued (21 Nov, the tags design was accepted; SSE4-tags-design.md, sections 4, 5.7 and 7): TG1 tag declarations plus a load check (first; reserve the group `trade` and the prefix `economy.`), TG2 physics wall, TG3 authored overrules derived, TG4 seeded chance on sift rules, TG5 tags level with facts in data views, TG12 dressing rules. One PR each. TG6 (the V3 importer, proven core thread) needs TG1 and TG3. Standing rule: a tag is only a label, never an input to physics.

Record design (PR 81, 21 Nov): the secrecy choices are ruled. Dice are keyed on the GM-only table secret, and `competitive` packs need sealed seats (rulings 7 and 8). Section 2.4 adds the kept kinds `stock` and `extracted` per resource, plus one effect, `transfer`: a single gate decision, refused when the source holds too little (TG11). Honesty and source marks are now called "marks"; "tag" means a label only.

Queued 21 Nov 18:49 (coordinator): matter pack, track MP (design /mnt/project-files/sse4/SSE4-matter-pack-design.md, repo PR 89). The kernel thread owns MP1 schema, MP1c constants, and MP2 data (with the citations thread). Alex has not ruled on its open questions, so build to the recommendations. Fit it in the order by judgement.

Y2 draw merged PR 152 (22 Nov 08:57): op `sky_from_here`, log handover/packages/Y2.md, fudge F35. G5 giants changed to 2 L_MS + 50 at 4,500 K (was 30 L + 50, sky 3-5x too full); counts from the Sun now 196/594/1,902 brighter than V 3/4/5 vs real 171/513/1,602. Speed lesson: `GalaxyEnvelope::local` is dear (metal-history scan); use `systems_pc3`/`gas_pc3`/`surface` for hot loops. Next on kernel thread: Y4.
Y3 glow merged PR 154 (light thread): sse_sky::glow::class_radii mirrors sky.rs's class radius rule (margin, probe temps, 4 kpc cap); change both together or glow and points double count.
22 Nov: Y4 part 1 merged PR 155 (galaxies in sky, F37). PR 157 merged: massive stars (MLR M^1.5 above 31 Msun, Humphreys-Davidson cool limit), 14 luminosity classes (renderer must read class edges, not hard-code indexes), per-class keyed slots, 6.5 sky 28 s -> 12.6 s; plus system `frame` on resolved_system/window_view (Sol measured from base-sol, others drawn, F38). Y4 part 2 satellites PR 158 (F39). Next: Y4 part 3 halo count tree.
Galaxy histories (track GH, design handover/design/SSE4-galaxy-histories-design.md, accepted 22 Nov 13:15): kernel thread owns GH0 timing (PR 171: orbits 8x dearer than estimated), GH1-GH7, DM2-DM5. GH1 replaces Y4 part 2's separate draw; bins stay keyed from psi 1e-3 so Chart members don't move. DM1/DM6 are the sheet thread's.
