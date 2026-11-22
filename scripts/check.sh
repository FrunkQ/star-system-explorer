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
python3 scripts/build-base-names.py --check
# MP22: no new typed constant or material number, and the ledger only shrinks.
python3 scripts/test-audit-matter.py >/dev/null
if git rev-parse -q --verify origin/main >/dev/null; then
  python3 scripts/audit-matter.py --ledger handover/packages/MP0-typed-copies.txt --base origin/main
else
  python3 scripts/audit-matter.py --ledger handover/packages/MP0-typed-copies.txt
fi
# The importer must still build without formation (the converter module).
cargo build -q -p sse-convert-wasm
echo "all checks passed"
