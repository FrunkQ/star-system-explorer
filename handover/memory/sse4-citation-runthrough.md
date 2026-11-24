---
name: sse4-citation-runthrough
description: Consensus run-through of every cited paper (track CR, thread "Verify every cited paper", 24 Nov 2026): master list, check, daily batches, how results come back
metadata:
  type: project
  modified: 2026-11-24T19:37:39.302Z
---
- Alex 24 Nov 2026: check every paper cited in a "Why" through Consensus (exists, says what we claim), background, about 30 a day, master list scored off.
- Repo: `handover/citations/master.jsonl` (truth, one paper per line keyed `surnameYEAR`), `MASTER.md` (generated), README (procedure for old and new account); `scripts/citations.py` (--add, --update, --check, --batches, --score), `scripts/test-citations.py`; CI job `citations` and check.sh fail on a pack/code citation not on the list. Status file `handover/status/citations.md`.
- Builds on the 19 Nov register `docs/citations.md` (statuses read into `earlier`) and Alex's visual polish Consensus pass (PR 464, scored as results-00: 8 confirmed, Rhines 1975 says something else).
- 1,465 papers: tier 1 from memory 71, tier 2 golden 109, tier 3 code 775, tier 4 design-only 510. Batches and replies in project files `sse4/citations/` (batch-NN.md, results-NN.md).
- Old account: Claude can't reach Consensus; Alex pastes a batch into a Claude with Consensus and brings the reply back. New account (25 Nov on): Claude runs the round itself.
- Wrong results go to the coordinator naming the owning thread; this track never edits other packs.

Related: [[sse4-citations-and-fudges]]
