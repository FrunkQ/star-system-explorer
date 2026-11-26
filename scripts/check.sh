#!/usr/bin/env bash
# Everything CI runs for Rust, in one go. Run before every commit.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo fmt --all
cargo clippy --workspace --all-targets -q -- -D warnings
cargo test --workspace -q
# The master data sheet must match its sources (scripts/build-base-reference.py).
python3 scripts/build-base-reference.py --check
# The catalogued Solar System must match its sources and agree with the sheet.
python3 scripts/build-base-sol.py --check
python3 scripts/build-base-starters.py --check
python3 scripts/build-base-names.py --check
# The class atlas reads the sheet and base-sol (its builder is what the MP22 audit reads).
python3 scripts/build-base-atlas.py --check
# The Traveller pack reads the sheet too (its builder is what the MP22 audit reads).
python3 scripts/build-traveller-pack.py --check
# The future tech base pack works its figures out from the sheet (FT1).
python3 scripts/build-base-tech.py --check
# The Paper spacecraft store pack is generated from its studies (RK6).
python3 scripts/build-paper-spacecraft.py --check
# MP22 closed: no typed constant or material number anywhere outside the sheet.
python3 scripts/test-audit-matter.py >/dev/null
python3 scripts/audit-matter.py --closed
# ER: the side-door ledger (edits-reach design 4 and 12) may only shrink. Convert a
# site to the physics in force, then run `python3 scripts/audit-side-doors.py --shrink`.
python3 scripts/test-audit-side-doors.py >/dev/null
if git rev-parse -q --verify origin/main >/dev/null; then
  python3 scripts/audit-side-doors.py --check --base origin/main
else
  python3 scripts/audit-side-doors.py --check
fi
# Every cited paper is on the citation run-through's master list (add new ones with --add).
# Warning only until Alex says the check may fail (and join CI).
python3 scripts/test-citations.py >/dev/null
python3 scripts/citations.py --check --warn
# The importer must still build without formation (the converter module).
cargo build -q -p sse-convert-wasm
echo "all checks passed"
