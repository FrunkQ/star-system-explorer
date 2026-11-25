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
thread also adds any it finds each day.

### Switching the check to fail (Alex's yes, 25 Nov 2026 01:27 UTC: "1-3 yes!", for "after the 25th")

Do this on the new account, on 26 Nov 2026 or later, in one pull request, never while the dev window is open:

1. Merge main, then run `python3 scripts/citations.py --add` so that main's own citations are all on the list, and
   confirm `python3 scripts/citations.py --check` passes (no `--warn`).
2. In `scripts/check.sh`, change `python3 scripts/citations.py --check --warn` to `python3 scripts/citations.py --check`
   and remove the "Warning only" comment line above it.
3. In `.github/workflows/ci.yml`, add this job just above `rust:` (it has no `needs`, so it runs on every pull request,
   docs-only and UI-only ones too):

   ```yaml
     # Every paper a pack or the code cites is on the citation run-through's master list (handover/citations/).
     citations:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4
         - run: python3 scripts/test-citations.py
         - run: python3 scripts/citations.py --check
   ```
4. Tell every builder thread before it merges: "a new citation needs `python3 scripts/citations.py --add` and the
   commit of `handover/citations/master.jsonl`, or CI fails".
5. Update this section and `handover/status/citations.md` to say the check now fails.

Keep adding your row to `docs/citations.md` with what you checked, as before: that register is the builders' own
record and this list reads its statuses into `earlier`.

## Where it stands and when it restarts (25 Nov 2026)

Batches 1 to 9 went out. Batches 1 to 5 came back whole; batches 6 to 9 came back in part, because Alex's
Consensus plan reached 100% of its month. **The plan resets on 1 December 2026: no new batch before then.**

- Done: 294 papers scored (219 confirmed, 63 not found, 6 say something else, 6 wrong citation). Tiers 1 and 2
  (from memory, and behind golden numbers) are finished except one tier 1 paper.
- Left: 1,231 unchecked, all tier 3 (code and packs, 712) or tier 4 (design notes only, 518), plus the one tier 1.
  The papers of batches 6 to 9 that got no reply line are still `unchecked` in `master.jsonl`, so they need no
  list of their own: the next `--batches` run picks them up first, by tier.
- Restart on or after 1 December: `python3 scripts/citations.py --batches <folder> --start 10 --size 50`, then the
  round for whichever account is in use (below). At 50 a day, about 25 days; a month's plan held about 400 papers,
  so expect one more pause before the end, around tier 4.
- Corrections waiting on owners: `sse4/citations/corrections-01.md` and `corrections-02-05.md` in the project
  files (sections B0, B, C). Batches 6 to 9 add theirs to the same table in the status file.

### The papers Consensus cannot check (the "not found" rows)

Most "not found" rows exist and match, but are books, reports, standards or data notes that Consensus does not
index, or the number sits in a paywalled text. Do not spend Consensus on them again. Route instead:

1. Reports with their own page (NIST technical notes and JANAF, NASA technical reports, USGS circulars, ESA and
   IAU documents): fetch the publisher's page or the PDF from ntrs.nasa.gov, nvlpubs.nist.gov, pubs.usgs.gov and
   the like, and check the figure in the text itself. These are free and usually full text.
2. DOIs: resolve them through doi.org (or the Crossref API) to confirm the title, authors, year and venue. That
   settles "exists and is cited right"; the claim still needs the text.
3. Books (Osterbrock, Rybicki and Lightman, Born and Wolf, Turcotte and Schubert, Pierrehumbert, Perryman,
   Skolnik, Gray, Liou and others): confirm the edition and chapter through the publisher or a library catalogue,
   and check the figure against a page Alex or a reader can open. Mark `confirmed` only with the page or equation
   number in the note; otherwise leave `not found` with "book, needs a page check".
4. arXiv preprints: the arXiv abstract page and PDF are free; check there.

Score these by hand with the same reply line format (`key | status | found | figure | note`, with "checked via
<route>" in the note) and run `--score` as usual.

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
