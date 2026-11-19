# Orbits at any time, the campaign clock and pack calendars

Status: done (19 Nov 2026). Kernel track, asked for by the chart room (UI thread) for human-scale time: the moons chart, travel and session play need bodies where they are on a given day, a campaign "now" that every seat agrees on, and the game system's own dates.

## What was built
`crates/sse-kernel/src/orbits.rs`, four kernel ops.

**`orbits_at {system, at_s: [..], bodies?}`** (1 to 64 times):
- Every body at or under `system` with `orbit.semi_major_axis` is placed at each time, relative to its host, in metres, in the host's reference plane (z = 0). It also gets its longitude (`phase_deg`, in [0, 360)) and its period.
- Facts read (the importer's names, `docs/v3-save-fields.md`):
  - the orbital elements `orbit.semi_major_axis`, `orbit.eccentricity`, `orbit.inclination`, `orbit.ascending_node`, `orbit.argument_of_periapsis` and `orbit.mean_anomaly_at_epoch`;
  - the host, from `orbit.host`, else `parent`, else the address's parent; the host's `body.mass` (plus the body's own mass, so mu = G(M + m)).
- Epoch: `orbit.epoch_s`, else an SSE 3 `orbit.epoch_v3_ms` read as milliseconds, else the universe origin.
- Kepler's equation is solved with 16 fixed Newton steps (from M, or from pi when e >= 0.8). That makes the result bit-identical on every device and accurate to 1e-12 rad up to e = 0.99.
- **Missing phase:** a body with no `orbit.mean_anomaly_at_epoch` takes a keyed draw on the universe seed, its address and stream `orbit.phase.v1`. It is labelled `phase_source: resolved`. The draw is the same on every call and every device, and the GM can pin it like any fact. With no universe open and inline bodies, there is no seed, so such a body is unplaced.
- **Never guessed:** a body missing its host mass, its semi-major axis, or an eccentricity below 1 comes back `placed: false` with `missing` naming each gap.
- `bodies: [{address, facts}]` places bodies the universe doesn't hold, for previews and imports before adoption.

**The campaign clock**:
- `set_campaign_now {at_s}` writes `time.campaign_now`, a universe event at the origin that sets `campaign.now_s` on the universe root. Every reading of the universe sees it, and seats get it with the rest of the universe. It may move back as well as forwards (a GM rewind).
- `campaign_now` returns the latest value and the event that set it (absent if never set).

**Pack calendars**, `provides.other.calendar`:
- Shape: `{name, epoch_s, epoch_year?, day_s, year_days, months?: [{name, days}], format}`. Unknown keys are refused.
- Checked at `load_stack`: positive day and year, a format, and months that add up to the year. A bad calendar rejects the stack with the problem named. The top pack's calendar wins.
- `calendar` returns it. The kernel carries no date formatting; the UI formats with `format`, e.g. `"{day:03}-{year}"` for Traveller's Imperial dates, or `"{d} {month} {year}"`.
- New fixture `tests/fixtures/packs/imperial-calendar.json` (365 days, no months, epoch year 1105).

## Checks
- Unit, `orbits::tests`:
  - Earth's period is 365.256 days, it is at periapsis at M = 0 and at apoapsis half a year on.
  - Kepler round trip to 1e-12 for e from 0 to 0.99.
  - Vis-viva holds along an inclined, eccentric orbit.
  - A missing phase is unplaced without a seed, and resolved and repeatable with one; a missing host mass is named.
- Kernel, `orbits_clock_and_calendar`:
  - the calendar loads, and months that don't fill the year reject the stack;
  - inline bodies place without a universe;
  - an adopted formation system (11 bodies, 8 days to 70 years) places every body with dated phases, the same twice;
  - the synthetic Sol import gives Earth 365.25 days and the Moon 27.28 days;
  - the clock is unset, set, moved back and read.
- Contract: 10 new lines (160 to 169). Lines 1 (`hello`) and 41 (the unknown-op list) list the new ops. The TypeScript types are regenerated.

Golden changes (on purpose): `contract.jsonl` lines 1 and 41, plus the new lines 160 to 169. No engine golden changed.

## Limits and next
- Two-body Kepler only. There is no precession, no n-body perturbation, and binaries are a primary plus its companion. Deep-time positions belong to the physics track's integrator, not this op.
- Positions are local to each host. The chart room nests them (moon in planet in star), as the rendering rules ask: local orbits per parent in f64.
- J4's `pairs` still read `pos.*` facts. A later pair source can read `orbits_at` instead, so that triggers follow bodies along their orbits.
- No date parsing. The UI turns typed dates into universe seconds with the calendar.

## Addition: named days, weeks and leap years (19 Nov 2026, asked for by the Traveller pack)
The calendar gains four optional fields; unknown keys are still refused.
- `named_days: [{day, name, outside_weeks?}]`: `day` is 1-based, within the year (one more when `leap` is set). `outside_weeks` needs a week with `continuous: false`.
- `week: {names, first_day?, continuous?}`: `first_day` is 1-based; `continuous` defaults to true (weeks run across years). The Imperial calendar uses first_day 2 and continuous false, with Holiday (day 1) outside the weeks.
- `leap: {every, except_every?, but_every?}`: `except_every` must be a multiple of `every`, `but_every` a multiple of `except_every` (and needs it). Gregorian is 4, 100, 400.
- `leap_month` (0-based): needed when both months and leap are set; refused without leap.

The kernel only checks the shape; the chart room formats dates with `packages/ui-contract/src/calendar.ts` (Traveller thread). Test: `orbits::tests::calendars_with_named_days_weeks_and_leaps`. No golden changed.
