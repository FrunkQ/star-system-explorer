# Status: Citation run-through

Owner thread: "Verify every cited paper". Covers: CR, the Consensus check of every paper cited in a "Why" (exists, and
says what SSE4 claims), the master list `handover/citations/master.jsonl` and its check. Procedure:
`handover/citations/README.md`. The per-area register `docs/citations.md` stays each builder's own record.

Only the owner thread edits this file. Index and standing rules: `handover/START-HERE.md`.

| Package | State | Log |
|---|---|---|
| CR1 master list and check: `scripts/citations.py` gathers author-year citations, DOIs and arXiv ids from packs, crates, packages, the worker and design notes; 1,465 papers in four tiers; `scripts/check.sh` warns on a pack or code citation not on the list (`--add` fixes it) | done (this PR); a failing check and its CI job wait on Alex's yes | handover/citations/README.md |
| CR2 the visual polish pass (Alex's Consensus run, PR 464) scored onto the list: 8 confirmed, Rhines 1975 "says something else" (π√(2U/β) is one jet's width; U must be the eddy speed) | done (this PR); raised to the coordinator | project files sse4/citations/results-00-visual-polish.md |
| CR3 daily batches of 30, tier 1 (marked from memory) first: batches 1 to 5 written to project files `sse4/citations/` | next: Alex carries batch 1 to a Claude with Consensus; from 25 Nov the new account runs the round itself | handover/citations/README.md |
