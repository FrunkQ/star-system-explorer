---
name: sse4-spaceflight-history
description: SSE4 track SH, Roll call (who was in space on any day), In memory and Names carried; accepted 25 Nov 01:20 UTC; SH1, SH2, SH6 merged (PR 510), SH7 part 1 (PR 527) and SH5 (PR 531) merged
metadata:
  type: project
  modified: 2026-11-25T05:24:44.535Z
---

Alex, 25 Nov 2026 01:20 UTC accepted the design (handover/design/SSE4-spaceflight-history-design.md, all 13 recommendations are rulings): "yes to all - stay restrained and respectful - one place we don't joke! :)". Status file handover/status/spaceflight-history.md; package logs handover/packages/SH1-SH2.md, SH6.md, SH7.md.

**Built (thread "Build roll call and In memory"):** SH1 ui-contract/rollcall.ts, SH2 shell/src/rollcall/ (Roll call on the system rail under Travel; pack read from `?spaceflight=<Sol today folder>`), SH6 In memory (ui-contract/inmemory.ts) all merged in PR 510 on 25 Nov 04:35 UTC. SH7 part 1 (namescarried.ts: memorials, Kepler orbitAt, "Names carried" text on In memory) PR 527, opened 25 Nov 05:30 UTC. Rail limit RAIL_MAX is 11 for now on Alex's word (01:59 UTC "overpack… rationalisation later"); go back to ten at the rationalisation. CI line tests/ui/rollcall.e2e.mjs was added with Alex's yes.

SH5 Records and On this day (rollrecords.ts, pure look-ups, not kernel sifters yet) merged PR 531 at 09:43 UTC 25 Nov.

Next: SH7 part 2 (chart layer needs a seam from the chart room; "tonight" needs constellation table and H magnitudes; naming dates from the pack thread), SH3 count strip (ruler is shared shell), SH4 a person's flights (manifest thread), SH8 front door opens Sol (chart room). Pack has h_mag per memorial (PR 529); no naming dates yet.

Rules of restraint, standing for anything touching real deaths (design 8.1): every name and date sourced twice, nothing invented, no spectacle, plain words, no decoration, never a hold backdrop or a joke. No invented monument in the real pack.

Local testing gotcha: never copy pack files from another branch into store-packs/ locally; a stray logs/index.json broke sse-nav sol_today_logs. The real files are on main since PR 506.

Data seam with the pack thread ([[sse4-real-spaceflight]]).
