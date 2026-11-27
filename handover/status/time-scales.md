# Status: one time pipeline for every scale (track TP)

Owner thread: "Check how time scales fit together" (Fable, audit and design only, 26 Nov 2026) for the note. TP1 builder: thread "Stop dates losing their seconds" (Opus, 26 Nov 2026), coordinator's word.
Design: `design/SSE4-time-scales-design.md`, **ACCEPTED by Alex 26 Nov 2026 23:27 UTC** ("i agree with all": the eight questions of its section 12), with his four rulings in its section 12A and `decisions.md`: the Big Bang is defined to the second so "now is now"; time is not capped (fine time in whole seconds, then a slower deep time to the end); the ruler ends in a break with late events as dated marks after it; everything runs to the end of time that way, galaxies included (the note's section 12B lists what still happens late). TP is a code only, never shown in the product.

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

Nothing is built. Golden notes are expectations. Released to build on Alex's yes: all of TP1 to TP9; TP10 needs its own short design first. Order: TP1 first and alone; then TP2 and TP4 side by side; then TP3, TP5 and TP9; then TP6; then TP7 and TP8. Core golden (`tests/golden/contract.jsonl`): TP2, TP5, TP8, and TP9 if `UniverseTime`'s stored form changes; TP1 is expected to move none.

| Package | State | Owner (proposed) |
|---|---|---|
| TP0 the audit, the note in `handover/design/`, this file, the row in START-HERE | done, 26 Nov | this thread |
| TP1 `Instant` in `sse-core` (`since`, `angle_since`, checked `from_years`); `orbits_at` on dates; the one-second test at 4.61 and 13.8 Gyr seen red first; shrink-only ledger `packages/TP1-float-dates.txt` (a cargo test). Log `packages/TP1.md`. TP1b: flares and the flight log on dates (one core golden line, `activity_read`); nav's own ops and the screens wait for TP3; prominences (PR 658) to follow | TP1 done 26 Nov (PR 661, no golden moved; PR 672 ledger catch-up); TP1b in review (PR 676) | kernel |
| TP2a first slice (RW17 part 2, thread "Make eclipses happen on time"): `present_s` stated in base-cosmology; `birth_s` reported by `resolved_system` for every system; the screens draw at `birth_s` + age; the Sol door opens on the device's present through `set_campaign_now` | built, see below | eclipses thread |
| TP2 the rest: `time.present_s` on the root, `system.birth_s` as a written fact, `dated`'s convention removed, Sol moved onto `present_s` (list below) | released; needs TP2a, TP1b, TP3 and the Sol today anchor mapping; core golden slot | kernel |
| TP3 the clock object `sse.clock/1`; the screens write `set_campaign_now`; `now_yr`, `campaign_now_yr`, `campaign_epoch_yr`, `clock_yr` become whole seconds; save migration | released; needs TP2; agree with CLK and three desks | chart room |
| TP4 process declarations (shape, own time, anchor) and the pace ladder; op `pace_ladder`; shrink-only check | released; needs TP1; area `time` | kernel |
| TP5 marches made path-free (grid from the star alone, full orbit state, checkpoints, exact-date write-back); two-legs test seen red first | released; needs TP1; core golden slot | kernel |
| TP6 `happenings`: one op for marks from lines, streams, kept look-aheads, voyages and rule ticks | released; needs TP4; area `time` | kernel |
| TP7 pacing on the ruler and the Run desk clock (Free, Catch, Stop at); natural paces by name; "what you passed"; the pace ladder strip; the ruler's break, the late marks after it and "next change". Mock-up to Alex first | released; needs TP3, TP6; with TM3b | chart room, three desks |
| TP8 follow, jump or window as one kernel rule; the supernova and fast nebula as jumps; the general window; the coordinator used | released; needs TP5; core golden slot | kernel |
| TP10 every scale's future: galaxy rings and merger trees run forward, the web's horizon, late streams at cited rates, black hole ends (the note's 12B) | needs its own short design first (Fable, with the cosmos and galaxy owners), then TP9 | cosmos, galaxy |
| TP9 time to the end: checked conversions; deep time past 292 billion years with one ordering; look-aheads and marks across the change-over; far eras as marks; until TP10, the galaxy past the present labelled and registered | released; needs TP1 (marks part needs TP6); core golden slot if the stored form changes | kernel |

## TP2a, as built (RW17 part 2, 26 Nov 2026)

Done:
- `packs/base-cosmology` states `present_s` = 435,494,880,000,000,000 beside `now_gyr` (ruling 12A.1); `sse-cosmos` checks it is whole and equals `now_gyr` in Julian years. Nothing reads it yet.
- `resolved_system` reports `birth_s` (whole seconds) for every system with a spec: its spec's birth year times the Julian year. A system read from facts whose star carries `time.j2000_s` now takes that date as its present (`spec_from_facts`), so Sol's birth year is J2000's year minus its age, not minus it from second 0.
- The session reads `birth_s` from the reply's digits (a double would be 32 s coarse). `Workspace.birth_s`; `drawnSecond(birth_s, years)` and `screenYears` in `ui-contract/src/sky-clock.ts`. The chart's orbits, the orbit box and Roll call draw at `birth_s` + the screen's age; pack calendars are read on the same axis (`fromPackCalendar(pc, birth_s)`).
- The Sol door, opened fresh on a clock nobody has set (kernel campaign now unset and no elapsed campaign time on the screen), opens on the device's present: UTC read onto TT with the cited leap-second list (`ui-contract/src/leap-seconds.json`, IERS Bulletin C), sent to the kernel's `set_campaign_now`. A saved campaign or a date the GM set is left alone. A real-sky system's calendar is Gregorian from its own anchor (`skyCalendar`), not a typed copy.
- `session.setCampaignNow` is now a `record` intent (`set_campaign_now`, at_s as exact digits), so players replaying the GM's log move their campaign now too.
- The chart says "As measured", "Plausible" or "Partly measured" from `orbits_at`'s `at[].motion`.
- Proof for the move onto `present_s`: `eclipse_on_time_at_the_universes_present` puts Sol's anchor at `present_s` and gets the 2024 eclipse to its second and 64 distinct places in 64 one-second steps for a measured and a plain body. Seen red (39 s late) with the date turned into a double before the subtraction.
- Browser tests that open the Solar System pin the device's date (`pinClock` in `tests/ui/nav.mjs`, 26 Nov 2026 12:00 UTC); press budgets unchanged.

Left for TP2 and after (not done here):
- Sol stays at 63,115,200,000 until TP1b (flight logs and flares on whole seconds), TP3 (the screens' clock in whole seconds: today a years double, about 30 s at 4.6 Gyr) and the Sol today pack's anchor mapping (its logs and calendar are typed at the 63e9 anchor). The move is then one edit in `scripts/build-base-sol.py` (`J2000_S` read from base-cosmology's `present_s`), plus reading the anchor fact exactly in the shell (`anchorOf` takes a safe integer only).
- `time.present_s` on the universe root and `system.birth_s` as a written fact; `dated`'s convention and `system.now_year` (old saves only).
- The kernel's clock still ages a system from its base age plus campaign time elapsed since the first `set_campaign_now`: Sol opened today counts as 4.5673 Gyr old today, not at J2000 (27 years apart).
- Other screens that turn the screen's years into kernel seconds directly (`yearsToSeconds(now_yr)` for notes, reveals, the cutaway's sweep, the surface's sun): still on the star-age axis until TP3.

## Asks of running threads (sent to the coordinator 26 Nov 2026)

- Eclipses (PR 649 part 2): anchor read from `time.j2000_s` only; the Sol door sets the date through the kernel's `set_campaign_now`.
- CLK: keep "seconds since the campaign epoch"; TP3 puts the kernel's clock under it.
- Binary mass transfer: one window over the pair, kept as lines; a supernova is a jump; write back all six elements with their date.
- Heat flow: thermal history is a march on the body's own grid from birth, kept as a line.
- Three desks TM3b: "what is due" and "jump to a date" come from `happenings` once TP6 exists.
- Everyone: no new absolute time in a double, no new `_yr` clock field.

## Changes to the note since TP0

- 26 Nov 2026, 21:17 UTC: the march rule (section 6, point 1) names a third fixed anchor, a measured state's own date, at the ask of the thread "Design melting inside worlds" (HF6, PR 666) with the coordinator's agreement. HF6.3 would share TP5's two-legs test harness.

## Questions

All eight answered yes by Alex, 26 Nov 2026 23:27 UTC; questions 3 and 4 changed by his rulings that time is not capped and that everything, galaxies included, runs to the end of time (the note's section 12A). None open.
