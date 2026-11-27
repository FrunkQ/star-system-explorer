# Fudge register

This register is kept as one file per entry in [`docs/fudges/`](fudges/), so two pull requests that each add an entry never
edit the same file (27 Nov 2026). Read it whole on one page at https://v4.starsystemx.com/fudges.html (built from
main at every deploy), or locally with:

    python3 scripts/registers.py --print fudges

**To add an entry:** in the table's folder under `docs/fudges/`, copy any row file to a new file named after the entry's
id or a short slug (for example `F-S37.md`), and edit its last line. Keep the header and separator lines as they
are. Prose and headings are the plain `.md` files beside the table folders. `scripts/check.sh` runs
`python3 scripts/registers.py --check`.

A branch from before the split that added rows to this file: take main's version of this file, put your copy aside,
and run `python3 scripts/registers.py --absorb fudges --from <your copy>`.

This file is written by `scripts/registers.py`; do not add rows here.
