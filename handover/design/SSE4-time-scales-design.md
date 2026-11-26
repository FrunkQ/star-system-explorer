# SSE4: one time pipeline for every scale (track TP)

Status: **PROPOSED, 26 Nov 2026. Not yet accepted by Alex.** Audit and design only; nothing here is built. The eight questions of section 12 are open, each with a recommendation. TP is a code only, never shown in the product.

Alex's ask, in his words (26 Nov 2026, 19:33 UTC): "I think one of the cleverest things you're doing here is managing all of our different time scales harmoniously together. So we can have galaxies working on their scale, solar systems working on their scale, stars working on their scale, and be able to know when to slow things down in the simulation and speed things up again. I think if that hasn't been thought through properly, we need to have a smooth pipeline where all that's managed together. Otherwise, we may end up tying ourselves in knots. It's one of those architecture points we need to make sure is solid as so much is going to lean on it. but that may be in place already".

Read with: `SSE4-design-handoff.md` (6.1 rule 6, 6.3, section 11 the coordinator), `SSE4-top-level-ui-and-onboarding.md` (12 and 17.1: one clock, accepted 21 Nov), `SSE4-conserved-chain-design.md` (sections 10 and 11: local sets and their beats), `SSE4-three-modes-design.md` (7.3: the Run desk's clock), `SSE4-real-catalogues-design.md` (6A.2: right place at the right time), `handover/packages/RW17.md` (PR 649), project memory `sse4-picture-clock` (track CLK).

---

## 1. The short answer

**It is half in place. The ideas are right and mostly built. The joining is not.**

What is solid (section 2): the kernel keeps time as whole seconds, most physics is a curve that can be read at any date without stepping there, short events come from draws keyed by place and time bin, deep futures run on a branch and are kept only on a yes, and "one clock for the whole universe" was accepted on 21 Nov.

What is not (section 3), each checked in the code on main at `152f4d3`:

1. **There are four zero points for time**, not one, and nothing states how they relate.
2. **The app's clock is not the kernel's clock.** The ruler's time lives in the screen's own state, in years. Nothing in the app ever calls the kernel's `set_campaign_now`. Views that ask the kernel for "now" get nothing and fall back to zero.
3. **Dates are held in floating point**, so near a star's present age a position can only change every 32 seconds, and every 64 seconds at the universe's present age. Measured: a station in low orbit jumps 424 km at a time; an Earth jumps 1,135 km.
4. **Ageing a system in two legs does not give the same answer as ageing it in one.** Measured on the Solar System: stop at 9 Gyr, keep, go on to 14 Gyr, and Earth's final eccentricity is 0.0086 where the single run gives 0.0199, and Mercury is swallowed 1.3 Myr earlier.
5. **Nothing looks for short events when time runs fast.** Play adds elapsed time to the date and draws there. A flare, an eclipse, a nova or a close pass between two frames is never seen.
6. **Nothing decides when to slow down.** There is no rule, no owner and no list of what is coming.
7. **Whole-second dates stop at 292 billion years.** Past that the conversion from years wraps round silently in the shipped build.
8. **Galaxies and the cosmic web stop at the present.** Stars age into the future; the galaxy round them does not.

None of this is a knot yet, because today each screen mostly looks at one scale at a time. It becomes one the moment things lean on it together: binaries aged as a pair, novae, prominences, cloud stirring, thermal histories, the Run desk's game clock and real eclipse dates are all arriving this week.

The design (sections 4 to 10) is small because it mostly names what the code already does well and makes it the only way:

- **One axis.** Universe time in whole seconds from the origin. Every other date (a star's age, a calendar date, J2000, a ship's log) is that axis read through a named anchor.
- **Floats hold only differences.** A date is an integer. Physics works on "seconds since my anchor", subtracted as integers first.
- **Three shapes of process, and no fourth.** A *line* (read at any date), a *stream* (events drawn per time bin), a *march* (stepped state, on a grid fixed by the thing itself, kept as a line).
- **One hand-over rule between scales**, by comparing how fast the driver changes with how fast the thing responds: follow, jump, or open a window and step them together.
- **Marks.** Every process can say what happens between two dates. The clock asks before it crosses.
- **The clock paces; the physics never does.** Slowing down is a choice the clock makes from the marks ahead and the natural pace of what is on screen. The universe at a date is the same however you got there.

---

## 2. What is in place and sound

| Piece | Where | Why it is sound |
|---|---|---|
| Universe time is whole seconds plus nanoseconds | `sse-chronicle/src/event.rs:14` (`UniverseTime { s: i64, ns: u32 }`) | Exact. 83 of the kernel protocol's time fields are `at_s: i64` |
| Life lines: a thing's whole life is drawn once and any date is a read of it | handoff 6.1 rule 6; `sse-formation` life line, `sse-time/src/star.rs` (`state_at`, `mass_at`) | Jumping to a date and stepping there are the same by construction |
| Galaxy history is stepped once from the start on fixed steps and kept | `sse-cosmos/src/rings.rs` (`evolve_clocked`, 10 Myr galaxy steps, 50 Myr ring steps) | Any date is a read of the kept history |
| Event streams keyed by address and time bin | `sse-cosmos/src/events.rs` (supernovae, bursts, close passes, 10 to 100 Myr bins); `sse-activity/src/lib.rs:676` (flares, bins from the star's own rate) | Any window can be asked without the windows before it |
| Epoch stepping with detailed windows | `sse-time/src/epoch.rs`: analytic steps, a fourth-order symplectic window on a phase change or a close pair, crossings dated by fixed bisection | The star's own clock sets the step (80 steps through the giant phase), not a fixed dt |
| Look-ahead as a branch, kept only on a yes; replay rebuilds it | `sse-time/src/branch.rs` | Nothing stepped is written unless accepted; work is counted in steps, never wall time |
| One campaign clock in the kernel, and "make this system this old, today" | `sse-kernel/src/clock.rs`, `orbits.rs:663` (`campaign.now_s`) | The accepted model of top-level design 12.2 |
| A coordinator that advances suites to common boundaries | `sse-suite/src/coordinator.rs` (H11) | Proven in its own crate (see 3.9: nothing uses it yet) |
| Rule ticks between two boundaries, refused if too many | `sse-trigger/src/lib.rs:902` ("N ticks in one step; step more often") | The one place that already says what to do when the step is too long |
| Calendars as pack data over the clock | `orbits.rs:702`, `ui-contract/src/calendar.ts` | A date is a view of the clock, not a second clock |
| Measured motion with an anchor per system, hand-over to a plain orbit outside its span (PR 649, open) | `time.j2000_s`, `at[].motion` | The right shape: a named anchor, and the reply says which model placed the body |
| The picture clock (track CLK, in progress) | `ui-contract/src/picture-clock.ts`, `ws.clock` in seconds since the campaign epoch | Seconds since an anchor is exactly the form section 5 asks for |

So the answer to "may be in place already" is: the hard half is. The kernel's physics is mostly stateless in time, which is the property that makes many scales possible at all.

---

## 3. What is not, with the evidence

Checked on main at `152f4d3`. Two findings were proved by temporary tests that were run and then removed; their output is quoted. I read the weather, surface, life and navigation crates by their time inputs only, not line by line.

### 3.1 Four zero points

| Who | Zero is | Unit | "Now" for the base app |
|---|---|---|---|
| The screens (`ws.now_yr`, `workspace.svelte.ts:6`) | the star's birth (`fate.ts:123` sends `birth_year: 0`) | years, floating point | 4.61e9 |
| Sol's data (base-sol, Sol today, nav fixtures, PR 649's `time.j2000_s`) | 2,000 Julian years before J2000 | seconds | 63,115,200,000 |
| The default calendar (`calendar.ts:252`, `GREGORIAN.epoch_s: 0`) | 1 January 2000 | seconds | 0 |
| The cosmos (`sse-cosmos`, `now_gyr` 13.8) and generated systems (`generated.rs:94`, `birth_year = born_gyr * 1e9`) | the Big Bang | Gyr or years, floating point | 13.8e9 years |

The kernel bridges the last two by a convention, not a fact: "the clock's first setting is the cosmology's present" (`sse-kernel/src/lib.rs:1351`, `dated`). A system read from facts with no `system.now_year` puts its present at the origin (`adopt.rs:777`). Switching systems on the chart re-bases the screen's clock to the new star's age and keeps "time elapsed" (`App.svelte:879`, `shiftCampaign`).

This is the cause of the Sol door symptom: the date shown is not the date the bodies are drawn at. PR 649 part 2 fixes that one pair. The general fix is section 4.

### 3.2 The app's clock is not the kernel's clock

- The kernel has `set_campaign_now` (`lib.rs:6966`) and derives every system's age from it (`clock.rs`).
- A search of `packages/` and `tests/` finds no caller of the session's `setCampaignNow`. The ruler's "Make this now" only sets `ws.campaign_now_yr` (`workspace.svelte.ts:40`). The clock is saved in the screen's part of the campaign save, in years (`campaign.ts`), and sent to players as `clock_yr` (`p2p.ts:52`).
- Seven views ask the kernel for now (`campaignNow()`): the globe's weather (`Body.svelte:354`), true air (`true-air.ts:660`), Aboard, Constructs, the sector sheets. With the clock never set they get nothing and use day 0 or second 0. So weather and clouds are drawn for a different date from the orbits on the same screen.
- The scene is re-derived only when the viewed time moves by 1,000 years (`App.svelte:877`); orbits are fetched every frame for the viewed time.

### 3.3 Dates in floating point

A double holds 53 bits. At 1.455e17 seconds (4.61 Gyr) the smallest step is 32 s; at 4.35e17 seconds (13.8 Gyr) it is 64 s; in years at 4.61e9 it is 30 s.

- `orbits.rs:104`: `self.m0 + n * (t as f64 - self.epoch)`. The integer date is turned into a double before the epoch is taken off.
- Probe (run, then removed): a station at 6,778 km round an Earth, asked for at 64 successive seconds.

```
j2000-style 6.3e10:  station 64 distinct positions, biggest jump 7.7 km;    Earth 64, 30.3 km
star age 4.61 Gyr:   station  3 distinct positions, biggest jump 423.6 km;  Earth  3, 1135.4 km
cosmic 13.8 Gyr:     station  2 distinct positions, biggest jump 423.6 km;  Earth  2, 2314.3 km
```

- The same holds wherever an absolute time is a double: flare times (`sse-activity/src/lib.rs:699`, seconds of the star's age), navigation (`sse-nav/src/lib.rs:556`, `t_s: f64`; `kernel/src/nav.rs:123`, `now_s` from `birth_year` and `age_myr`), the screens' `now_yr`, weather's `day: f64`.
- Sol works today only because its zero point is 2,000 years back. Moving Sol onto the true axis without fixing this would break eclipses that PR 649 has just made exact to the second.

### 3.4 Ageing in two legs differs from ageing in one

- The epoch stepper's main-sequence grid starts at the date asked from (`epoch.rs:228`), its white dwarf grid depends on the date asked to (`epoch.rs:249`), and a kept look-ahead writes back only the size, eccentricity and tilt of each orbit, at the last step before the date, not at the date (`adopt.rs:688`, `aged_event`). The turning angles and the phase are dropped.
- Probe (run, then removed): the Solar System aged to 14 Gyr in one run, and again stopping at 9 Gyr with only what the write-back keeps.

```
one run:   Earth e=0.019870511  Mercury engulfed at 12728.416165 Myr
two legs:  Earth e=0.008643527  Mercury engulfed at 12727.130594 Myr
```

- Both runs are repeatable, so determinism in the narrow sense holds. What fails is the stronger rule the handoff asks for: the universe at a date should not depend on which dates you stopped at on the way.
- `branch.rs:109` also writes `orbit.a`, `orbit.e`, `orbit.i` (in AU and degrees) that nothing in the kernel reads; the real write-back is `aged_event`.

### 3.5 Short events at high speed

Play is `t += elapsed * step` once a frame, sent to the views ten times a second (`TimeRuler.svelte:192`). At a million years a second, one frame is 16,000 years. Flares, eclipses, novae and close passes exist in the kernel as dated things, but no code asks "what happened between the last frame and this one". The ruler shows only the events a view hands it up front.

### 3.6 Slowing down

There is no code for it and no design for it. The words "slow down" appear nowhere in the handover's time sections. TM3b (not started) plans "what is due" for the Run desk only.

### 3.7 The range of whole seconds

`i64` seconds reach 2.92e11 years. `UniverseTime::years` multiplies without a check (`event.rs:26`), and the release profile has no overflow checks (`Cargo.toml:39`), so a look-ahead on a red dwarf (life about 1e13 years) dates its events at wrapped, negative times. Arithmetic, not run. Far-future eras are listed in log years (`sse-cosmos/src/epochs.rs`) and labelled speculative, which is fine; they are just not on the axis.

### 3.8 Galaxies stop at now

`cosmos.rs:268` clamps a galaxy read to `[t_start, now]`; the galaxy's structure caps the scale factor at 1 (`cosmos.rs:389`); `rings.rs` types 13.8 in three places (the delay-time weights). A system aged 5 Gyr ahead sits in a galaxy held at today. That is a fair simplification, but it is not written down as one.

### 3.9 Coupling

- The coordinator (H11) is used by no kernel code; only `sse-life` implements its socket, as a stub.
- Orbits widen with the star's mass loss by the slow rule (`a * M` constant, `epoch.rs:334`) whatever the rate. A planetary nebula thrown off inside one step, or a supernova, is treated as a slow wind. (Reported by the binary thread; confirmed here.)
- Each star of a close pair is aged alone and the rule table is consulted afterwards (`binary.rs`).

---

## 4. One axis, with anchors

**Universe time `T` is whole seconds from the origin, with nanoseconds, as `UniverseTime` already is.** It is the only absolute time anywhere: kernel, packs, saves, screens, players.

Everything else is `T` read through an **anchor**, and an anchor is a fact with an address like any other:

| Anchor | Fact | Set by |
|---|---|---|
| The universe's present when it was opened | `time.present_s` on the universe root | the cosmology's `now_gyr`, once, as a whole number of seconds |
| The campaign's now | `campaign.now_s` (exists) | the GM; starts at `time.present_s` |
| A system's birth | `system.birth_s` | resolve; replaces `birth_year` and `system.now_year` |
| A real-sky system's reference date | `time.j2000_s` (PR 649) | the pack, as an offset from the present (question 2) |
| A calendar's first day | `calendar.epoch_s` | the pack, or the campaign |
| A body's dated phase | `orbit.epoch_s` (exists) | resolve or a catalogue |

Then:

- A star's age is `T - birth_s`. The ruler's "System time" is that, shown; it is not a second clock.
- "Make this system this old, today" moves `birth_s`. "Make this the campaign's now" moves `campaign.now_s`. Both exist in the kernel; the screens start calling them.
- The cosmos reads `T` as Gyr by dividing. The convention in `dated` goes.
- A pack with its own time scale (Sol today's logs, a real catalogue) states one anchor ("my second X is the system's reference date") and the kernel maps it. Pack data is not rewritten.

**Range.** Whole seconds reach 292 billion years. Dated events beyond that are refused with a plain sentence and the conversion is checked, never wrapped. Reading a life line at a great age still works, because an age is a difference, not a date. Whether dated time should go further is question 3.

---

## 5. Floats hold only differences

The rule: **an absolute time is never a floating-point number.** A date is integers. Physics works in "seconds since my anchor", and the subtraction is done in integers before anything becomes a double:

```
dt = (T.s - epoch.s) as f64 + (T.ns - epoch.ns) * 1e-9     // then n * dt, and so on
```

- Kernel: one small type in `sse-core` (`Instant`, `since(anchor) -> f64`, checked `from_years`), and it is the only way to turn a date into a number. `orbits_at`, navigation, flares, spin and weather take an `Instant` and their own anchor.
- Screens: time is a `bigint` of seconds. A view that animates holds `anchor + offset`, as the picture clock already does. `now_yr` goes; years are a display.
- Long spans lose phase anyway (a period known to 1 part in 1e9 says nothing about phase after 1e9 orbits). That is honest and already labelled "plausible". What must not happen is a date that cannot move by one second.
- Kept honest by a check that only shrinks, as the figures ledger does: a count of `at_s as f64` and of `_yr` clock fields per file that may go down, never up; and a test that one-second steps at 13.8 Gyr give 64 distinct positions.

---

## 6. Three shapes of process, and no fourth

Every piece of physics that depends on time is one of these. A new piece says which, in its block's declaration.

| Shape | What it is | Read at any date | Examples today |
|---|---|---|---|
| **Line** | A curve fixed when the thing was resolved. A pure function of `T - anchor` | yes, directly | a star's life, white dwarf cooling, a Kepler orbit, spin angle, a measured-motion series, the galaxy's kept history, calendar dates |
| **Stream** | Events drawn from the seed, keyed by address and time bin. The bin length comes from the process's own rate | yes: any window, without the windows before it | flares, supernovae and bursts near a system, close passes, impacts |
| **March** | State that truly depends on its own past, stepped | only through the rule below | orbits through a star's mass loss and tides, binary mass transfer, a world's thermal history, galaxy rings, life |

**The march rule.** A march is allowed only if it ends up as a line:

1. It starts from a fixed anchor: the subject's birth, or a checkpoint recorded in the chronicle.
2. Its grid is a function of the subject alone (the star's own phases, the body's own cooling time), never of the date asked from or to.
3. What it works out is kept as a table against time, with the whole state at each grid point (for orbits: all six elements and their date, not three).
4. A read between grid points uses a stated rule (the analytic step from the grid point before).
5. A kept look-ahead writes checkpoints; a later run starts from the nearest one and lands on the same grid.

With that, "jump to a date" and "step there in any number of legs" give the same bits, and section 3.4's test passes. Two things already work this way: the galaxy's rings and the ageing star's mass table.

**Every process declares**, as data beside its block (cited, editable, like any other block setting):

- its shape;
- its **own time** `tau`: a formula from its physics for how fast it changes (an orbital period, a convective turnover time, a cooling time, a crossing time). Never a typed constant;
- its anchor;
- what **marks** it can publish (section 8);
- for a march, its grid rule.

The list of these is the **pace ladder**: one table of every process and its own time, from flare seconds to galaxy gigayears, read from the stack. It is the block's "UI element that makes sense" for time, and it is what the clock uses in section 9.

---

## 7. Handing over between scales

"Every level sits in the one above." For time that means:

**Down.** A parent gives its child lines and streams, never a half-stepped state: the galaxy gives a system its birth date, place and the stream of passers-by; the star gives its planets `L(t)`, `M(t)`, `R(t)`; a planet gives its surface the insolation and tide curves.

**Up.** A child changes its parent only through dated events at a boundary (mass thrown out, a body ejected, a flux of gas). This is the coordinator's rule (H11) and the conserved chain's rule (whatever leaves a local set is a real object on a path).

**Across: follow, jump, or window.** When one process drives another, compare the driver's own time `tau_d` with the responder's `tau_r`:

| Case | Rule | Example |
|---|---|---|
| `tau_d` much longer than `tau_r` | **Follow.** The responder is a line of the driver (the adiabatic rule) | an orbit under a slow wind: `a M` constant |
| `tau_d` much shorter than `tau_r` | **Jump.** A dated event with a before and an after, worked out from conservation at that instant | a supernova's mass loss: the orbit keeps its velocity and its new shape follows; it is unbound if more than half the pair's mass goes |
| comparable | **Window.** March the named set together on one step, `min(tau) / N`, from a checkpoint to a checkpoint | a close encounter (built); a planetary nebula against a wide orbit; Roche-lobe overflow in a pair; a world's interior regimes against its heat loss |

The thresholds are pack numbers with a citation (for mass loss: Hadjidemetriou 1963, already cited in `epoch.rs`; Veras et al. 2011, MNRAS 417, 2104, to be added to the master list and checked: Consensus is out until 1 Dec 2026, so treat it as unverified until then). The window is not new: `epoch.rs` already opens one for close pairs. This makes it the one general mechanism for anything coupled, with the set, the span and the step chosen by the rule above. The coordinator's boundary loop is how a window with a suite in it (life) runs.

This one rule is the answer to three of this week's threads:

- **Binaries** are a window over the pair: both life lines, the orbit and the transfer marched together, kept as lines for each star and the orbit. Novae and flares on them are streams whose rates read those lines. When one "goes BOOM" it is a jump.
- **Thermal history** is a march on the body's own grid from birth, kept as a line; its regime changes are marks.
- **Prominences, cells and cloud stirring** are streams and lines with seconds-to-days `tau`. They never slow the system; they are simply not resolved unless the clock is near their pace (section 9).

---

## 8. Marks: what happens between two dates

One kernel op, the same for every scale:

```
happenings { scope, from_s, to_s, at_least?, limit? }
  -> { marks: [{ at_s, lasts_s, kind, subjects, rank, why, source }], summarised: [{ kind, count, biggest }] }
```

- A **line** finds its marks by root-finding on the curve (the fixed bisection `epoch.rs` already uses): a phase change, an eclipse or transit (from the orbit lines), a periapsis, a crossing of a threshold.
- A **stream** returns its draws in the window.
- A **march** returns the transitions in its kept table.
- Campaign things use the same op: a voyage's arrival, a rule's tick, a note's date.
- `rank` is the sifter's significance score (handoff 6.6), so "interesting" is measured, not typed.
- When a window holds too many (a red dwarf's flares over a million years), the reply gives a count and the biggest, as `sse-activity` already does (`summarised_count`). Nothing is silently dropped.
- Cost is bounded: each process answers for its own window from its own shape; nothing is stepped to find a mark.

This is also the honest base for Alex's "what is interesting here" list, for the ruler's ticks, for the Run desk's "what is due" (TM3b), and for the sifters.

---

## 9. The clock, and who slows it down

**One clock object** for the session, `sse.clock/1`:

```
{ now_s,            // the campaign's now: the kernel's campaign.now_s
  view_s,           // the date being looked at (the ruler); equals now_s unless looking elsewhere
  rate,             // universe seconds per second of play; 0 is stopped
  pace }            // "free" | "catch" | "stop-at"
```

- The kernel owns `now_s`. The session owns `view_s`, `rate` and `pace`. Every view reads time from this object and nowhere else. Players receive `now_s` as whole seconds.
- The picture clock (CLK) is this object's `view_s` and `rate` seen by the drawing code.

**The physics never slows down.** The universe at a date is a fact. Pacing is presentation: the wall clock only chooses which dates get shown. So play at any frame rate, on any device, shows the same thing at the same date, and determinism is untouched.

**Who decides: the clock, from two inputs the kernel gives it.**

1. **The marks ahead.** While playing, the clock asks `happenings` for the stretch it is about to cross (a few seconds of play ahead), at or above the rank the GM has set.
2. **The natural pace of what is on screen.** Each view names the processes it draws; from the pace ladder the clock knows the rate at which each is watchable (its `tau` over a few seconds).

**Three ways to play:**

| Pace | What it does | Default on |
|---|---|---|
| **Free** | Runs at the set rate. Marks pass as ticks on the ruler with a count ("3 eclipses, 1 flare storm"). Anything faster than the pictures is drawn where it is and named in the note (CLK's rule) | Build |
| **Catch** | When a mark at or above the chosen rank is coming, ease the rate down to that mark's natural pace, show it, name it, then ease back up. "Slow down here, something is happening" | Explore |
| **Stop at** | Run until the next mark at or above the rank, and stop there with it named | Run (the GM's "what is due", "go to then") |

- The rate control offers the natural paces of what is in view by name ("a day a second: watch the moons"; "10 Myr a second: watch the arms"), so the common choice is one press. A typed rate stays possible.
- A jump (dragging the ruler, "jump to a date") asks `happenings` for the stretch skipped and says what it passed over. On the Run desk that is the dialog TM3b already plans.
- Easing is on the rate, with the interface style's motion rules; a stopped clock is a still picture.

Nothing here is a new screen. It is one control on the ruler and on the Run desk's clock, and one strip (the pace ladder) reachable from it.

---

## 10. What a new piece of physics must do to plug in

1. Take time as an `Instant` and an anchor. No absolute doubles, no wall clock.
2. Say its shape: line, stream or march. If march, give its grid rule and keep its table.
3. Give its own time `tau` as a cited formula.
4. Publish its marks through `happenings`, with a `why`.
5. If something else drives it, name the driver; the follow, jump or window rule is then applied by the kernel, not written again in the block.
6. Pass the two standing tests: read at a date equals stepped to that date in two legs (bit for bit); one-second steps at 13.8 Gyr are distinct.
7. Appear on the pace ladder (it does, from 2 and 3).

A piece that cannot do these is a Big Block and says why in `docs/fudges.md`, as today.

---

## 11. Packages

Sizes: S under a day, M one to two days, L more. Golden notes are expectations; a builder confirms before queueing.

| Package | What | Needs | Golden | Size |
|---|---|---|---|---|
| **TP1** | `Instant` in `sse-core`: `since`, checked `from_years`, range refusal. `orbits_at`, flares, spin and nav's body positions subtract in integers. The one-second test at 13.8 Gyr, seen red first. The shrink-only ledger of `at_s as f64` in `scripts/check.sh` | nothing | none expected at today's small Sol dates; core lane if a line moves | M |
| **TP2** | The anchors: `time.present_s` on the root, `system.birth_s`; `dated`'s convention removed; `system.now_year` read on old saves only; generated and adopted systems on the one axis. Sol's reference date per question 2 | TP1; after PR 649 parts 2 and 3 | core (system facts gain a field) | M |
| **TP3** | The clock object: `sse.clock/1` in the session; the ruler and the Run desk clock write `set_campaign_now`; `now_yr`, `campaign_now_yr`, `campaign_epoch_yr` and `clock_yr` replaced by whole seconds; the seven `campaignNow()` readers read the clock; save migration | TP2; agree with CLK and the three desks thread | none (screens) | L |
| **TP4** | The process declaration and the pace ladder: shape, `tau`, anchor per time-dependent block, as pack data; op `pace_ladder`; a check that a time-dependent op without a declaration fails (ledger that only shrinks) | TP1 | area `time` | M |
| **TP5** | Marches made path-free: the epoch stepper's grid from the star alone; full orbit state at each grid point; checkpoints; `aged_event` at the exact date; the dead `orbit.a` fields removed. The two-legs test, seen red first | TP1 | core (look-ahead lines), one slot | M |
| **TP6** | `happenings`: lines (phase changes, eclipses, periapses), streams (flares, close passes, supernovae), kept look-aheads, voyages and rule ticks behind one op, with ranks from the sifter and the summarised form | TP4 | area `time` | L |
| **TP7** | Pacing: Free, Catch and Stop at on the ruler and the Run desk clock; natural paces by name; "what you passed" on a jump; the pace ladder strip. Mock-up to Alex first | TP3, TP6; with TM3b | none | M |
| **TP8** | Follow, jump or window as one kernel rule: mass loss by rate (the supernova and the fast nebula as jumps), the general window over a named set, the coordinator used for windows with a suite | TP5 | core, one slot | L |
| **TP9** | Honest ends: checked range in every conversion; the galaxy and web past the present labelled and in the fudge register (or built, question 4) | TP1 | none | S |

Order: TP1 first and alone (it protects PR 649's eclipses). Then TP2 and TP4 side by side. TP3 and TP5 next. TP6, then TP7 and TP8.

**Asks of threads already running** (sent to the coordinator, 26 Nov):

- *Make eclipses happen on time* (PR 649 part 2): go ahead as planned. Read the anchor from `time.j2000_s` only and never type the number again, as agreed; and have the Sol door set the date through the kernel's `set_campaign_now`, not only the screen's state, so the weather and the orbits agree. TP2 may later move the number; one place makes that one edit.
- *Run all animation off the clock* (CLK): the picture clock's "seconds since the campaign epoch" is the form section 5 asks for. Keep it; TP3 gives it the kernel's clock underneath.
- *Draw binary mass transfer and flares*: age the pair in one window and keep each star and the orbit as lines (section 7); treat a supernova as a jump; write back all six orbit elements with their date.
- *Fix low heat flow in worlds*: the thermal history is a march on the body's own grid from birth, kept as a line, with regime changes as marks (section 6). Do not start its grid at "now".
- *Build the three desks* (TM3b): take "what is due" and "jump to a date" from `happenings` when TP6 exists rather than building a second list.
- *Everyone*: no new absolute time in a double, and no new `_yr` clock field.

---

## 12. Open questions for Alex

1. **Is the one axis right: whole seconds from the Big Bang, with a star's age, calendar dates and J2000 all read through named anchors?** It was accepted in principle on 21 Nov ("one clock"). *Recommend yes.*
2. **Where does the real Solar System sit on that axis?** *Recommend: J2000.0 is the universe's present (`time.present_s`, 13.8 Gyr after the origin) for a universe whose home is the real Sol, so "today" is 26.9 years after it.* Sol's pack data keeps its own numbers and is mapped by the one anchor. Existing saves of the Sol door would move; you have said existing universes can be thrown away.
3. **Should dated time go past 292 billion years?** A red dwarf's whole life (10 trillion years) can already be read as a life line and drawn in the star gallery. What cannot be done is put a dated event out there in a campaign. *Recommend no for now: refuse with a plain sentence, keep the far eras as the labelled list they are.* Widening later is a contained change.
4. **Should galaxies and the cosmic web run past the present?** Today they are held at now while stars age on. *Recommend: label it now (TP9), and build the galaxy's future (the next big merger, star formation winding down) as its own design later. It is a good story for a GM but nothing leans on it yet.*
5. **Default pacing by desk: Catch on Explore, Free on Build, Stop at on Run?** *Recommend yes*, with the rank at which Catch slows down set to "big things only" (a star's phase change, an engulfment, a close pass, a nova, an eclipse of the world in view) until you say otherwise.
6. **When play is too fast to see something, is a tick and a count enough, or should the picture show a trace (a flash where the flare was)?** *Recommend tick and count now; traces per kind later as renderer work, each still drawn from a real mark.*
7. **Track code TP and one owner thread for TP1 to TP3 (kernel), with TP7 going to the chart room and three desks threads?** *Recommend yes*; routine, the coordinator may decide.
8. **May TP5 and TP8 change the Solar System's look-ahead figures?** Making the stepper path-free and the nebula a jump where it is one will move some dates and final orbits in the core golden. That is fixing the cause. *Recommend yes, one core slot each, reasons in the package logs.*

---

## 13. What this note does not cover

- Relativity for fast ships (ship clocks are in the exotic objects design) beyond noting that a ship's own time is one more anchor and a line.
- Light-time (the Moon's 1.3 s that is worth 37 s of eclipse, RW17): an observer's "now" is a retarded read of the same lines. It fits the model and belongs to the shadows thread.
- Performance budgets for `happenings`; TP6 measures them.
- A line-by-line audit of weather, surface, life and navigation. Their time inputs were read; their insides were not.
