# Citation run-through (track CR)

Alex, 24 Nov 2026: check every paper cited in a "Why" through Consensus, both that it exists and that it says what
SSE4 claims, as a background task spread over days to fit the Consensus budget ("a few dozen a day"), with a master
list that is scored off as papers are confirmed. Owner thread: "Verify every cited paper". Status:
`handover/status/citations.md`.

## The files

| File | What |
|---|---|
| `master.jsonl` | **The master list**, one paper per line, sorted by key. The source of truth. |
| `MASTER.md` | The same list as tables in order of work, with counts. Written by the script; never edit by hand. |
| `../../scripts/citations.py` | Gathers, checks, writes batches, scores replies (`--help`). |
| `../../scripts/test-citations.py` | Tests of what counts as a citation. |
| Project files `sse4/citations/` | Daily batches `batch-NN.md` and the replies `results-NN.md` (working files, not in the repo; the scores and notes they carry are copied into `master.jsonl`). |

Each line of `master.jsonl`:

| Field | Meaning |
|---|---|
| `key` | first author's surname folded to a-z, then the year (and letter): `shakura1973`, `kopparapu2013a`. A paper cited only by DOI is `doi:10....`. This is the name used in batches and replies. |
| `citation` | the fullest form as written in SSE4 (a hint; the places are the truth) |
| `doi`, `arxiv` | every DOI and arXiv id written beside it |
| `marks` | marks found beside it: "from memory", "to verify", "unverified", "not read here" ... |
| `golden` | it appears in a golden test output (`tests/golden/`), so a golden-tested number rests on it |
| `in_code` | cited in packs, crates, packages or the worker (not only in design notes) |
| `earlier` | the weakest status the first pass gave it in `docs/citations*` (19 Nov onward) |
| `tier` | order of work, below |
| `status`, `checked`, `found`, `note` | **the score**: status word, date and how, the full citation found, the key figure and what differs |
| `places`, `claims` | every file (and block id) that cites it, and up to six distinct sentences that rest on it |
| `manual` | true for a hand-added row (a source with no author-year form, such as Kaye and Laby); kept by `--update` |
| `gone` | true when nothing cites it any more; kept so a score is never lost |

Status words: **unchecked**, **confirmed** (exists as cited and supports the claim), **wrong citation** (exists, but
a year, journal, DOI, author or equation number is wrong), **says something else** (does not support the claim, or
gives another number or formula), **not found**.

## Order of work

1. **Marked from memory or to verify** (about 70): the visual polish note's "[from memory]" list, the exogeology and
   galaxy-histories "found (from memory, check)" rows, kernel comments marked "to verify", and Kaye and Laby for the
   two gas refractivities still open from the light pack's four (MP2).
2. **Behind golden-tested numbers** (about 110): cited in a golden output.
3. **Other packs and code** (about 775).
4. **Design notes only** (about 510).

Inside a tier: papers the first pass left unverified (or never saw) first, then the most-cited. `--batches` follows this.

## The rule for every builder

When a pack, the kernel, the UI or the worker cites a paper that is not on the list, `scripts/check.sh` prints a
warning (`python3 scripts/citations.py --check --warn`). Fix: `python3 scripts/citations.py --add`, then commit
`handover/citations/master.jsonl`. `--add` only adds the new papers as unchecked and touches no other line, so two
pull requests rarely conflict; if they do, take both sides (lines are independent and sorted by key). The run-through
thread also adds any it finds each day. Making the check fail, in check.sh and as a CI job, waits on Alex's yes (CI
edits are his call; a new failing check turns main red for every open PR until each adds its papers).

Keep adding your row to `docs/citations.md` with what you checked, as before: that register is the builders' own
record and this list reads its statuses into `earlier`.

## Daily round: on the old account (until 25 Nov 2026)

Claude on that account cannot reach Consensus, so Alex carries the batches.

1. Alex opens `sse4/citations/batch-NN.md` in the project files, pastes everything below its line into a Claude
   conversation that has the Consensus connector, and pastes the whole reply back into the citation thread.
2. The thread saves the reply as `sse4/citations/results-NN.md`, then runs
   `python3 scripts/citations.py --score <that file> --date <YYYY-MM-DD>` and `--update`, and opens a small pull
   request with `handover/citations/` (merges when green).
3. Every paper scored other than `confirmed` is raised to the coordinator with the owning thread named (from the
   places and the track table in `handover/START-HERE.md`). The run-through never edits another thread's pack.
4. The next batch: `python3 scripts/citations.py --batches /mnt/project-files/sse4/citations --start <N+1>`.

## Daily round: on the new account (from 25 Nov 2026)

There Claude reaches Consensus itself, so one thread (or a daily scheduled task) runs the whole round:

1. `python3 scripts/citations.py --batches <a scratch folder> --start <N>` writes the next 30 unchecked papers.
2. For each paper in the batch, search Consensus (title words, authors and year, or the DOI), read the abstract or
   text, and write one reply line exactly as the batch asks: `key | status | full citation found | key figure | note`.
   Do not guess; "abstract only" goes in the note. Save the lines as `results-NN.md` (project files if the account
   has them, otherwise the scratch folder; the scores themselves land in `master.jsonl`).
3. Score, update, pull request and raise the wrong ones exactly as steps 2 and 3 above.
4. About thirty papers a day keeps inside the Consensus budget; 1,465 papers is about seven weeks. Tiers 1 and 2 are
   done in the first week.

If Consensus cannot be reached, stop the round and tell Alex; do not fall back to scoring from memory.

## What is not gathered

Only author-year citations, DOIs and arXiv ids are found. Software licences, specifications (WGSL, IEC sRGB), data
services (NIST, Horizons) and books cited by title alone are not papers and are left to the owning threads; one of
them that matters can be added by hand as a `manual` row. A key that holds two papers (same first author and year)
shows two DOIs; split it by hand into `name2011` and `name2011b` rows and fix the citing text's year letter.
