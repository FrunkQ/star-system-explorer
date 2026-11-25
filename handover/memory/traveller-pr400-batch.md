---
name: traveller-pr400-batch
description: Traveller thread's PR 400 (six packages) MERGED 25 Nov 2026 08:51 UTC as 67a2bf82; what it holds and where its contract cases live
metadata:
  type: project
  modified: 2026-11-24T22:17:30.133Z
---
PR 400 (branch claude/project-thread-mzdt73) holds X3j resources on the books, X3k mining and resource labels (op `mine`, pack sse.base-resources), X3l sector routes as links (import_sector `metadata`), RK12 Traveller weapons (provides.other.weapons, fudge F60), WBH part 1 (sse-codes `value` segments, sheet traveller.extended) and WBH part 2 (sse-codes::roll, op `roll_profile`, program traveller.social from the Cepheus SRD). Logs in handover/packages/; row in handover/status/traveller.md.

Golden: three refusal cases (transfer, mine, roll_profile) in the traveller area file, and the traveller-codes.json pack hash.

MERGED 25 Nov 08:51 UTC (67a2bf82). Contract cases live in tests/contract/areas/traveller.json (golden tests/golden/areas/traveller.jsonl); the Traveller area merges in parallel with other areas. Fudge entry is F60.

WBH part 3 (PR 550, merged 25 Nov 10:13 UTC as 0d12f7fc): "Roll every world…" on the Sector desk; rolled programs may name `decided_by` (traveller.social uses provenance.code) so imported worlds keep their whole profile, bases included.

**Why (history):** the golden lane (coordinator, 24 Nov 21:19): 468, 406, 417, the kernel's golden split part two, then 400. After part two, check with the kernel thread whether appended lines still need to wait.
**How to land:** merge origin/main, `SSE_BLESS=1 cargo test -q -p sse-kernel contract`, `SSE_BLESS=1 cargo test -q -p sse-codes golden` if the pack changed, export_bindings, scripts/check.sh, build-wasm, ui:check, ui:test, push, merge on green. Related: [[traveller-wbh-generator]], [[traveller-pack-asks-23nov]].
