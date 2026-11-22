# DM6: constants with a history (time laws)

Track GH, design `handover/design/SSE4-galaxy-histories-design.md` section 3.2 ("the bone"), accepted by Alex
on 22 Nov 2026 13:15. Built by the master data sheet thread.

## What was built

- **A `law` on any measured constants row** (`crates/sse-reference/src/laws.rs`):
  - `{"kind": "constant"}` is the default for every row.
  - `{"kind": "drift", "per_yr", "anchor_gyr"}` is a steady drift: value(t) = v (1 + per_yr (t - anchor) 1e9),
    with t in Gyr after the origin and v the row's value at the anchor.
  - `{"kind": "step", "at_gyr", "before"}` gives `before` until the date and the row's value from then on.
- **A law is never real.** The measured drift of G is zero to about a part in 10^13 a year (lunar laser
  ranging, Williams, Turyshev and Boggs 2004), so every law that is not constant must carry `mark` "authored"
  or "fictional" and its own `src` on the sheet. The reader refuses a law with neither.
- **A law goes on a measured row, never on a worked-out one.** The worked-out rows follow their inputs at each
  date, so Ωm drifts if Ωc does.
- **Reading by date** (`Reference`):
  - `constant_on(id, t_gyr)` gives one value at a date;
  - `has_history(id)` says whether a row or anything it rests on has a law;
  - `handle_on(t_gyr)` gives the one constants handle at that date.
  
  With no laws in the stack, `handle_on(t)` equals `handle()` bit for bit at every date, so nothing that exists
  today changes.

## Not here

- **Dated kernel calls** are the kernel thread's work: each resolve stage sets `handle_on` at its own date, and
  a star's ESTIMATED mark follows its life when G moves. DM6 gives the reader. The design names G as the first
  constant and the kernel's first user; any constant may carry a law.
- **Showing the history** on the Reference screen (a small graph by the row, and the mark) is the chart room's
  work, alongside MP10.

## Checks

- `constants_with_a_history_read_by_date` (sse-reference): drift and step at and around their dates. A law is
  refused with no mark, an unknown kind, a missing term, an unknown source, or on a worked-out row. A derived row follows a drifting
  input. With no law, `handle_on` equals `handle` at every date.
- `a_drifting_g_changes_the_year_by_keplers_factor_at_each_date` (kernel): the fixture
  `tests/fixtures/packs/reference-drifting-g.json` (fictional, G 10% stronger 1 Gyr ago) runs through the real
  `sse.orbit.period` atom. Today's year is the real one, and 1, 0.5 and 5 Gyr ago the year shrinks by exactly
  1/√(G ratio), checked to 1e-12. A stack with no law gives the same year at every date.

Goldens: none change.

## Also in this package

- `randall2008` is now verified on the sheet (ApJ 679, 1173 matches on Crossref). It was rate-limited in DM1.
