# Rulings during the build

This register is kept as one file per entry in [`handover/decisions/`](decisions/), so two pull requests that each add an entry never
edit the same file (27 Nov 2026). Read it whole on one page at https://v4.starsystemx.com/decisions.html (built from
main at every deploy), or locally with:

    python3 scripts/registers.py --print decisions

**To add an entry:** in the table's folder under `handover/decisions/`, copy any row file to a new file named after the entry's
id or a short slug (for example `2026-11-27-shared-lists.md`), and edit its last line. Keep the header and separator lines as they
are. Prose and headings are the plain `.md` files beside the table folders. `scripts/check.sh` runs
`python3 scripts/registers.py --check`.

A branch from before the split that added rows to this file: take main's version of this file, put your copy aside,
and run `python3 scripts/registers.py --absorb decisions --from <your copy>`.

This file is written by `scripts/registers.py`; do not add rows here.
