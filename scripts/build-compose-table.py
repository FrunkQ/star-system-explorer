#!/usr/bin/env python3
"""NM1: writes crates/sse-names/src/compose_table.rs, the canonical
composition pairs for Latin, Greek and Cyrillic letters, from Python's
unicodedata. Run it again only when moving to a newer Unicode."""
import unicodedata, sys

RANGES = [(0x0000, 0x024F), (0x0370, 0x03FF), (0x0400, 0x052F),
          (0x1E00, 0x1EFF), (0x1F00, 0x1FFF)]
pairs = []
for lo, hi in RANGES:
    for c in range(lo, hi + 1):
        d = unicodedata.decomposition(chr(c))
        if not d or d.startswith('<'):
            continue
        parts = [int(x, 16) for x in d.split()]
        if len(parts) != 2:
            continue
        # Composition exclusions recompose to something else under NFC.
        if unicodedata.normalize('NFC', chr(parts[0]) + chr(parts[1])) != chr(c):
            continue
        pairs.append((parts[0], parts[1], c))
pairs.sort()
out = [
    "//! Generated from Python's unicodedata (Unicode %s) by" % unicodedata.unidata_version,
    "//! `scripts/build-compose-table.py`: canonical pairs (base, combining mark) and the",
    "//! letter they compose to, for Latin, Greek and Cyrillic. Sorted by pair.",
    "",
    "pub const PAIRS: &[(u32, u32, u32)] = &[",
]
out += ["    (0x%04X, 0x%04X, 0x%04X)," % p for p in pairs]
out += ["];", ""]
sys.stdout.write("\n".join(out))
