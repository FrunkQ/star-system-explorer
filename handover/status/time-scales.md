# Status: one time pipeline for every scale (track TP)

Owner thread: "Check how time scales fit together" (Fable, audit and design only, 26 Nov 2026) for the note. TP1 builder: thread "Stop dates losing their seconds" (Opus, 26 Nov 2026), coordinator's word.
Design: `design/SSE4-time-scales-design.md`, **PROPOSED 26 Nov 2026, not yet accepted by Alex.** Its eight questions (section 12) are open, each with a recommendation. TP is a code only, never shown in the product.

What it is, in one line: the kernel's handling of many time scales is half in place (whole-second universe time, life lines read at any date, event streams keyed by time bin, look-ahead branches, one accepted clock), but the joining is not; this track makes one axis with named anchors, keeps floats to differences, allows three shapes of process (line, stream, march), gives one hand-over rule between scales (follow, jump, window), one op for "what happens between two dates" (marks), and one clock that paces play from those marks.

## The audit's findings (main at `152f4d3`, 26 Nov 2026)

Sound, with where: the note's section 2.

Not sound, each with file and line in the note's section 3:

1. Four zero points for time (the screens' star age in years; Sol's 2,000 years before J2000; the default calendar's 1 Jan 2000; the cosmos's Big Bang), bridged by a convention in `dated`.
2. The app never calls the kernel's `set_campaign_now`. The ruler's clock lives in the screen's state in years; seven views that ask the kernel for now get nothing and use zero.
3. Absolute times in floating point. Proved by a temporary test (removed): at a star's age `orbits_at` gives 3 distinct positions in 64 one-second steps (a low station jumps 424 km, an Earth 1,135 km); at 13.8 Gyr, 2.
4. Ageing in two legs differs from one. Proved by a temporary test (removed): the Solar System to 14 Gyr, against stopping at 9 Gyr with what the write-back keeps: Earth's final e 0.0199 against 0.0086; Mercury engulfed 1.3 Myr earlier. Cause: the stepper's grid depends on the dates asked from and to, and `aged_event` keeps three of six orbit elements at the step before the date.
5. Nothing finds short events when play runs fast.
6. Nothing decides when to slow down.
7. Whole seconds stop at 292 billion years and `UniverseTime::years` wraps unchecked in the release build (arithmetic, not run).
8. Galaxies stop at the present while stars age on; not in the fudge register.
9. The coordinator (H11) is used by nothing in the kernel; mass loss is always treated as slow; close pairs are aged one star at a time.

Not audited line by line: weather, surface, life, navigation (time inputs read only).

## Packages

Nothing is built. Golden notes are expectations.

| Package | State | Owner (proposed) |
|---|---|---|
| TP0 the audit, the note in `handover/design/`, this file, the row in START-HERE | done, 26 Nov | this thread |
| TP1 `Instant` in `sse-core` (`since`, `angle_since`, checked `from_years`); `orbits_at` on dates; the one-second test at 4.61 and 13.8 Gyr seen red first; shrink-only ledger `packages/TP1-float-dates.txt` (a cargo test). Log `packages/TP1.md`. TP1b: flares and the flight log on dates (one core golden line, `activity_read`); nav's own ops and the screens wait for TP3; prominences (PR 658) to follow | TP1 done 26 Nov (PR 661, no golden moved); TP1b in review | kernel |
| TP2 the anchors (`time.present_s`, `system.birth_s`); `dated`'s convention removed; Sol's reference date per question 2 | proposed; needs TP1, and PR 649 parts 2 and 3 merged; core golden slot | kernel |
| TP3 the clock object `sse.clock/1`; the screens write `set_campaign_now`; `now_yr`, `campaign_now_yr`, `campaign_epoch_yr`, `clock_yr` become whole seconds; save migration | proposed; needs TP2; agree with CLK and three desks | chart room |
| TP4 process declarations (shape, own time, anchor) and the pace ladder; op `pace_ladder`; shrink-only check | proposed; needs TP1; area `time` | kernel |
| TP5 marches made path-free (grid from the star alone, full orbit state, checkpoints, exact-date write-back); two-legs test seen red first | proposed; needs TP1; core golden slot | kernel |
| TP6 `happenings`: one op for marks from lines, streams, kept look-aheads, voyages and rule ticks | proposed; needs TP4; area `time` | kernel |
| TP7 pacing on the ruler and the Run desk clock (Free, Catch, Stop at); natural paces by name; "what you passed"; the pace ladder strip. Mock-up to Alex first | proposed; needs TP3, TP6; with TM3b | chart room, three desks |
| TP8 follow, jump or window as one kernel rule; the supernova and fast nebula as jumps; the general window; the coordinator used | proposed; needs TP5; core golden slot | kernel |
| TP9 checked range everywhere; the galaxy past the present labelled and registered | proposed; needs TP1 | kernel |

## Asks of running threads (sent to the coordinator 26 Nov 2026)

- Eclipses (PR 649 part 2): anchor read from `time.j2000_s` only; the Sol door sets the date through the kernel's `set_campaign_now`.
- CLK: keep "seconds since the campaign epoch"; TP3 puts the kernel's clock under it.
- Binary mass transfer: one window over the pair, kept as lines; a supernova is a jump; write back all six elements with their date.
- Heat flow: thermal history is a march on the body's own grid from birth, kept as a line.
- Three desks TM3b: "what is due" and "jump to a date" come from `happenings` once TP6 exists.
- Everyone: no new absolute time in a double, no new `_yr` clock field.

## Changes to the note since TP0

- 26 Nov 2026, 21:17 UTC: the march rule (section 6, point 1) names a third fixed anchor, a measured state's own date, at the ask of the thread "Design melting inside worlds" (HF6, PR 666) with the coordinator's agreement. HF6.3 would share TP5's two-legs test harness.

## Open questions for Alex

The note's section 12, eight questions: the one axis; where the real Solar System sits on it; dated time past 292 billion years; galaxies past the present; default pacing by desk; ticks or traces for events too fast to see; the track code and owner; whether TP5 and TP8 may move the Solar System's look-ahead figures.
