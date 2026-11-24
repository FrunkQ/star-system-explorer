---
name: sse4-binaries
description: SSE4 binaries and barycentres: all five rulings accepted by Alex 24 Nov 2026 and what the kernel built (BC1/K13)
metadata:
  type: project
  modified: 2026-11-24T16:50:00.000Z
---
Alex (24 Nov 14:50): binaries caught SSE3 out badly when added late; adding and removing bodies must be "well managed chaos". SSE3 used an 8% mass rule.

Accepted by Alex, all five (24 Nov; 2 to 5 on 16:41 "yes to all"):
1. Barycentre always: Jacobi hierarchy, no threshold.
2. No barycentre objects in the data: the point is computed and drawn; SSE3 `bary:N` nodes map onto it on import (not built yet).
3. Removing a body leaves its moons round its host, keeping their motion; "remove them too" is the other choice. Every add or remove reruns the stability verdict.
4. Charon and the big moons in base-sol now (Galileans, Titan, Triton, Phobos, Deimos; plus Pluto's small moons).
5. Binary evolution (mergers, mass transfer) later as dated changes, after the run-through.

Built by the kernel thread (branch claude/project-thread-dzkz4p, log handover/packages/BC1-barycentre-always.md): orbits_at gives r_m (from host centre), sys_m, centre_m; stability uses host plus inner mass; NAV uses G(M+m); hosts checked on write (bad_host); `rehost` op (keep motion or tidy); `withdraw` others = leave (default) / take / hand; `preview_tree` op; starter systems Alpha Centauri and Kepler-16 in packs/base-starters (Kepler-16 B and binary orbit still marked unverified).

**How to apply:** UI undo of add or paste sends withdraw with others "take"; the editing handles' "remove" uses the default. See [[sse4-editing-design]].
