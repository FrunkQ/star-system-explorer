# Status: Citation run-through

Owner thread: "Verify every cited paper". Covers: CR, the Consensus check of every paper cited in a "Why" (exists, and
says what SSE4 claims), the master list `handover/citations/master.jsonl` and its check. Procedure:
`handover/citations/README.md`. The per-area register `docs/citations.md` stays each builder's own record.

Only the owner thread edits this file. Index and standing rules: `handover/START-HERE.md`.

| Package | State | Log |
|---|---|---|
| CR1 master list and check: `scripts/citations.py` gathers author-year citations, DOIs and arXiv ids from packs, crates, packages, the worker and design notes; 1,465 papers in four tiers; `scripts/check.sh` warns on a pack or code citation not on the list (`--add` fixes it) | done (this PR); a failing check and its CI job wait on Alex's yes | handover/citations/README.md |
| CR2 the visual polish pass (Alex's Consensus run, PR 464) scored onto the list: 8 confirmed, Rhines 1975 "says something else" (π√(2U/β) is one jet's width; U must be the eddy speed) | done (this PR); raised to the coordinator | project files sse4/citations/results-00-visual-polish.md |
| CR4 batch 1 scored (25 Nov): 21 confirmed, 1 wrong citation, 4 say something else, 4 not reachable in Consensus; 16 corrections passed to owners | done (this PR) | project files sse4/citations/results-01.md, corrections-01.md |
| CR5 make the check fail (check.sh and a CI job) | Alex said yes on 25 Nov for after the 25th; ready as steps in handover/citations/README.md ("Switching the check to fail"), for the new account on 26 Nov or later | handover/citations/README.md |
| CR6 batches 2 to 9 scored (25 Nov): 294 papers scored in all, 219 confirmed, 63 not found, 6 say something else, 6 wrong citation; tiers 1 and 2 done; corrections for owners in project files `sse4/citations/corrections-02-05.md` and `corrections-06-09.md` | done (PR 500); raised to the coordinator | project files sse4/citations/results-02.md to results-09.md |
| CR3 daily batches, tier order: batches 1 to 9 written | PAUSED: Alex's Consensus plan hit its monthly limit during batches 6 to 9 and resets 1 December 2026. Restart then with batch 10 (README, "Where it stands"); 1,231 unchecked, all tier 3 and 4 but one | handover/citations/README.md |
