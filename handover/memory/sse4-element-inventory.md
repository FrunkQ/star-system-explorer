---
name: sse4-element-inventory
description: Every element's abundance and placement per body (affinity rows, MP7, MP6, MP8, patchy r-process draw): accepted 23 Nov 2026, waits for MP22 sweep and ND3b
metadata:
  type: project
  modified: 2026-11-23T12:16:54.214Z
---
Alex, 23 Nov 2026 12:16 UTC: "build them as stated, use blocks - best wait until all that work has completed" (the reference-data rewrite: MP22 typed-copies sweep and the kernel's ND3b atoms into Equation blocks, same area).

Accepted plan (matter pack design section 5), as visible blocks, in this order:
1. Per-element affinity rows on the sheet (iron-loving, rock-loving, crust-loving, volatile partition shares): owner "Build the master matter data sheet".
2. MP7: star inheritance to every element (Sun's Lodders 2021 abundance scaled by the element's origin-process shares and the star's level of each): owner kernel thread.
3. MP6: body inventory `read_composition` (elements ride in formation's six material groups; placed in core/mantle/crust/sea/air weighted by how far the world separated): owner proven core.
4. MP8 deposits (= TG11 inventory): surface ore processes enrich crustal share; totals conserved across detail levels: owner proven core.
Plus a deterministic per-region draw for neutron-star-merger elements (patchy gold).

**Why:** Alex asked whether all elements follow from the tracked processes and whether a collision can leave a Psyche-like core; the violent stage already strips mantle first (Asphaug 2010, Marcus 2009), but nothing joins elements to bodies.
**How to apply:** do not start until MP22 reaches 0 and ND3b is done. Then the data sheet thread starts part 1 and hands MP7 to the kernel and MP6/MP8 to proven core, citing Alex's message cmsg_01Ro269tjZuRP3p7iovTN7qNHJKD5Y3p1GtbGNXrNeEsX6. Related: [[sse4-matter-pack]], [[sse4-matter-build-state]].
