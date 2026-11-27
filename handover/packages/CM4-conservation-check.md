# CM4: the conservation check for compounds

Track CM. Design: `handover/design/SSE4-compound-ships-design.md` section 5.4 and 13. Built 27 Nov 2026 by the thread "Build mounts for compound ships", after CM1 and CM2.

## The check

`crates/sse-kernel/src/conservation_tests.rs`, one test. A fleet of 8 constructs, 3,000,000 kg in all, lives through 600 steps of captures (all five places, ports, stations, stand-offs), releases, splits (by part and by mass), docks, undocks and merges, written through the kernel's own ops from a fixed number stream, for three seeds. Refused steps are skipped; the test asserts each rule was really made at least 3 times per seed.

At every event's second, and half a minute after each:

1. the live constructs' `construct.mass_kg` sum to 3,000,000 kg within 1 kg, as an absolute number of kilograms (engine map PHY-34), and the failure says how many were made or lost;
2. no construct holds less than nothing;
3. reading the compound from every root counts every live construct exactly once and gives the same mass.

## Seen red

- One kilogram added at each split: fails at the first split, "the fleet holds 3000001 kg; it started with 3000000 kg (1 kg made)".
- **It found a real fault on its first run.** RS4's dock did not refuse a loop, so two craft could dock to each other; neither was then a root, and the compound reads lost them (6 constructs and 2,767,137 kg seen of 8 and 3,000,000). A dock now refuses with "cannot dock to what is docked to it" (`split.rs`), with its own assertion in `mount_tests.rs`. With that fix removed the check goes red again.

## What it does not cover yet

Propellant drawn by burns and amounts crossing a mount are CM3. Today a construct's propellant is inside its `construct.mass_kg` and moves only by a split or a merge, so this check already covers it. When CM3 lands, the sum gains "plus every burn's recorded propellant since the start" and the no-tank-below-zero check, as design 5.4 says; the test is built so those terms add to `check`.

No golden moves.
