---
name: sse4-citation-runthrough
description: Consensus run-through of every cited paper (track CR, thread "Verify every cited paper", 24 Nov 2026): master list, check, daily batches, how results come back
metadata:
  type: project
  modified: 2026-11-24T19:37:39.302Z
---
- Alex 24 Nov 2026: check every paper cited in a "Why" through Consensus (exists, says what we claim), background, about 30 a day, master list scored off.
- Repo: `handover/citations/master.jsonl` (truth, one paper per line keyed `surnameYEAR`), `MASTER.md` (generated), README (procedure for old and new account); `scripts/citations.py` (--add, --update, --check, --batches, --score), `scripts/test-citations.py`; check.sh warns on a pack/code citation not on the list (see below). Status file `handover/status/citations.md`.
- Builds on the 19 Nov register `docs/citations.md` (statuses read into `earlier`) and Alex's visual polish Consensus pass (PR 464, scored as results-00: 8 confirmed, Rhines 1975 says something else).
- About 1,530 papers (25 Nov): tiers from memory, golden, code, design-only. Batch 1 scored in PR 500. Batches and replies in project files `sse4/citations/` (batch-NN.md, results-NN.md).
- Old account: Claude can't reach Consensus; Alex pastes a batch into a Claude with Consensus and brings the reply back. New account (25 Nov on): Claude runs the round itself.
- Check is warning-only (check.sh --check --warn, no CI job). Alex said yes 25 Nov 01:27 UTC to making it fail in CI after the 25th: steps in handover/citations/README.md "Switching the check to fail", for the new account on 26 Nov or later.
- Batch 1 (30) cost Alex 7% of a day's Consensus budget; batches 2 to 5 are 50 each. The run-through thread fixes citation-only wording itself (coordinator's ruling 25 Nov), except strings captured in goldens or physics, which go to owners.
- PAUSED 25 Nov 2026: Alex's Consensus plan hit its monthly limit during batches 6 to 9; resets 1 Dec 2026. Restart with batch 10 (README "Where it stands"); 294 scored, 1,231 unchecked (tier 3 and 4). Non-papers (books, NIST and NASA reports) go by publisher page, doi.org or arXiv, not Consensus. Owner corrections: project files sse4/citations/corrections-*.md.
- Wrong results go to the coordinator naming the owning thread; this track never edits other packs.

Related: [[sse4-citations-and-fudges]]
