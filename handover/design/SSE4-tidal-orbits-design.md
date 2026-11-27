# SSE4: the orbit side of tidal heat (track HF, package HF6)

Status: **ACCEPTED by Alex, 26 Nov 2026 ("Yes to all, make sense!", 23:33 UTC).** His answers and two notes are in section 10A and win where they differ from the text. Written as a proposal at the coordinator's word earlier the same day. Design only; nothing here is built. HF is a code only, never shown in the product.

Read with: `SSE4-melt-and-heat-design.md` (sections 4.4 and 4.5, which this note completes), `SSE4-time-scales-design.md` (sections 6 to 8; PR 651, proposed), the measured-motion work of RW17 part 1 (PR 649; section 4.2 here says how the two live together), `SSE4-system-formation-design.md` (package F7, "long life: slow orbital change, tides", which this note is the moons' part of), `SSE4-ring-and-belt-editing-design.md` ("more ring mass = destroy a moon"), `crates/sse-formation/src/leftovers.rs` (how moons are made), `crates/sse-surface/src/lib.rs` (`tidal_flux`, `lock_time_gyr`, `TidalSpan`).

**On sources.** The paper search is out until 1 Dec 2026. Every paper figure is from memory and **unverified** (section 12). The two central formulas (sections 3.2 and 3.4) are derived here from conservation of energy and angular momentum, so they do not rest on a paper; the hand checks in section 5 use measured numbers quoted from memory.

---

## 1. The short answer

1. **Tidal heat is paid for by the host's spin, and the kernel has no account of that.** A moon raises a tide on its planet. The planet's spin drags that bulge ahead, which pushes the moon outward and slows the planet. That push is the only lasting source of energy for tidal heat. Today the kernel gives a moon an eccentricity and heats it from that for ever, with nothing paying.

2. **A stretched orbit dies away unless something holds it.** Io's own heating would round its orbit in about 150,000 years. It stays stretched because Europa and Ganymede keep step with it. A moon with no partner in step ends up on a round orbit and cold, however it started. Generated moons get a random eccentricity up to 0.01 and no partners (`leftovers.rs:513`), so every generated moon is heated by an orbit that could not last.

3. **For moons held in step, the heat follows from the host alone.** Conservation of energy and angular momentum give the heat a locked set must shed, with no rock or ice numbers in it (section 3.4). By hand, from measured values:
   - Io, Europa and Ganymede: 6.5 × 10¹³ W, against a measured 0.9 to 1.0 × 10¹⁴ W for Io.
   - Enceladus and Dione: 1.0 GW with the old value for Saturn (the published figure is 1.1), and 8 GW with the newer measured one, against a measured 10 to 16.

   That is within a factor of two for both, where the interior-only route of the melt design is ten to a hundred times low. The interior then decides the eccentricity the moon settles at, not the heat.

4. **One number may cover every giant planet.** Jupiter's and Saturn's measured tides both move their main heated moons outward on a time of about one to three times the system's age (0.8 and 1.7 by hand). That is what "resonance locking" predicts (Fuller et al. 2016). If it holds, a generated giant needs no dissipation number of its own. I recommend it as the law for giants, with the measured values winning for Jupiter and Saturn (question 2).

5. **It also gives the kernel real moon histories.** Moons move outward at a rate the same sum gives (the Moon: 3.7 cm a year by hand, measured 3.8). Pairs catch each other in step at a date. A moon inside the point where it orbits faster than its planet spins falls inward, and ends as a ring. The planet's day lengthens. Each is a dated event a GM can use, from the same numbers.

6. **It is five packages** (section 8). The first two give real moons their balance and need nothing from formation. Making generated moons catch each other in step changes what formation produces, so it is a core-lane change and the one to weigh (question 1).

---

## 2. What the code does today (main at `320ccf7`)

| Piece | Where | State |
|---|---|---|
| Generated regular moons | `sse-formation/src/leftovers.rs:440` | 2 to 5 per giant, the first at 4 to 8 planet radii, each 1.5 to 2.0 times further out than the last, eccentricity a random 0 to 0.01. No resonances. The comment on `a_planet_radii` says "tides move them, F7"; F7 is not built |
| Impact moons | `violent.rs:1002`, `leftovers.rs:524` | At their birth distance, a few planet radii |
| Resonances | `growth.rs:145`, `violent.rs:227` | Planets only (chains from migration, most broken later). `stability.rs` reads them. Nothing for moons |
| Moon heat | `sse-surface/src/lib.rs:430`, `tidal_flux` | From the orbit's eccentricity as given, k2/Q = 0.3/100 for every world, for all time |
| Past heating | `Body.tidal_history` (`TidalSpan`) | Authored only (Ganymede's fixture). The comment says "formation fills it when F makes captures" |
| Locking | `lock_time_gyr` | One Q and k2 for every world |
| The host's tide | nowhere | No host k2/Q, no torque, no change of a moon's distance or a planet's day |
| Real moons | `packs/base-sol` | Measured orbits. Their period ratios show the resonances; nothing reads that |

Generated moons are not given surface envelopes at all yet (`bodies_of` maps planets only), so today the fault shows on the Solar System, on fixtures and on imported systems.

---

## 3. The model

Every equation here is standard tide theory for small eccentricity. Each becomes one formula block (section 7).

### 3.1 The host's push

A moon of mass `m` at distance `a` from a host of mass `M`, radius `R_h` and tidal response `(k2/Q)_h`:

```
torque  T = (3/2) (k2/Q)_h · G m² R_h⁵ / a⁶ · sign(Ω_h − n)
drift   (da/dt)/a = 2 T / L,      L = m √(G M a)        (the moon's angular momentum)
```

`Ω_h` is the host's spin rate and `n` the moon's orbital rate. Outside the distance where the two match, the moon moves out and the host's day lengthens. Inside it (Phobos), or orbiting backwards (Triton), the moon moves in.

For one moon alone this integrates in closed form, so it is a **line**, not a march:

```
a(t)^(13/2) = a₀^(13/2) + (13/2) · 3 (k2/Q)_h (m/M) R_h⁵ √(G M) · t
```

The host's spin follows from keeping the total angular momentum (host's spin plus all moons' orbits) fixed.

### 3.2 A lone moon's orbit rounds off

The heat made in a moon (the existing `sse.tides.eccentricity_heating` block) comes out of its orbit at fixed angular momentum, which rounds the orbit:

```
P = (21/2) Im k2 · G M² R⁵ n e² / a⁶
e(t) = e₀ · exp(−t / τ_e),      τ_e = m a² n² e² / P = (2/21) (m/M) (a/R)⁵ / (Im k2 · n)
```

(Derived from the orbit's energy at fixed angular momentum. The host's tide adds a small term that raises `e`; it goes in the block and matters for the Moon.)

By hand: Io 150,000 years; the Moon 12 Gyr. So the Moon keeps its 0.055 and Io could not keep anything without help. Both are right.

**Rule:** a moon with no partner in step has today's eccentricity `e₀ exp(−age/τ_e)`, where `e₀` is what formation gave it, and its tidal heat follows that down. `τ_e` uses the moon's own Im k2 from its thermal line (the melt design's section 4), so a soft warm moon rounds off faster.

### 3.3 Catching a partner

Two moons whose period ratio is closing on a simple ratio (2:1, 3:2, 4:3, 5:3) are caught in step when they reach it, if the approach is slow. Inner moons are pushed out faster than outer ones (the torque falls as a⁻⁶), so orbits close up from the inside and capture is the usual outcome.

- **Converging and slow:** captured. The pair, or a longer chain, then moves out together with its period ratio fixed.
- **Diverging, or too fast:** they pass through. Each gets a kick of eccentricity that then dies away by 3.2. That is a dated pulse of heat: the kind of event thought to have made Ganymede's grooves and Miranda's cliffs.
- Capture chance and the kick's size are published laws for first-order resonances (Murray and Dermott 1999, chapter 8; Dermott, Malhotra and Murray 1988). I cannot write them reliably from memory. First build: capture is certain when converging with eccentricity under 0.02 and never otherwise, and a passage kicks the eccentricity to 0.01. Both are registered stand-ins until the chapter is read.

Whether a set is in step **today** is read off the period ratios (within 1 per cent of a simple ratio and both held there by the history). It is a label read off results, never stored as a fact that feeds physics.

### 3.4 The balance for a set held in step

For moons `i` of a locked set, with torques `T_i`, orbital rates `n_i` and angular momenta `L_i`:

```
drift of the whole set    (da/dt)/a = 2 ΣT_i / ΣL_i
heat the set must shed    H = Σ n_i T_i − ΣT_i · Σ(n_i L_i) / ΣL_i
```

Derivation, in three lines. The set keeps its period ratios, so every `a_i` grows at one rate. Angular momentum: the torques raise `ΣL_i`, and `L ∝ √a`, which gives the drift. Energy: the host's spin does work `Σ n_i T_i`; the orbits take up `−ΣE_i · (da/dt)/a` with `−E_i = n_i L_i / 2`; the rest is heat. Nothing in it depends on what the moons are made of.

`H` is the most the set can shed for long, and what it does shed once it has settled. It is shared among the members by their own heating at the eccentricities the resonance forces on them. The innermost takes nearly all of it (heating falls as a⁻⁶). First build: the split follows each member's `Im k2 · R⁵ n e² / a⁶` with measured eccentricities for real moons, and equal eccentricities for generated ones (a registered stand-in until the forced pattern is read from Murray and Dermott).

### 3.5 Heat sets the eccentricity, not the other way round

With `H_i` known for a member, the melt design's thermal line takes it as that moon's tidal heat (the same door as a measured pin: its section 4.4). The line gives the interior's state and so its Im k2. Then:

```
e_i = √( H_i / ( (21/2) Im k2_i · G M² R_i⁵ n_i / a_i⁶ ) )
```

- A stiff cold moon needs a large `e` to shed `H`; a soft warm one a small `e`.
- If the `e` needed is more than the resonance can hold (first build: 0.1, registered), the set is **not in balance**: the moon sheds only what it can at that limit, the orbit keeps stretching, and the card says so. In nature this is where a moon heats, softens, rounds off and cools again in cycles of some 100 Myr (Ojakangas and Stevenson 1986). The cycles are not modelled; the mean is.
- For real moons the measured `e` stays the fact. The balance is shown beside it ("Io sheds 2.2 W/m²; Jupiter can feed it 1.5; it is running a little hot and its orbit is rounding").

### 3.6 Where the host's response comes from

| Host | First build | Why |
|---|---|---|
| Jupiter, Saturn | measured k2/Q on the sheet (1.1 × 10⁻⁵; 1.6 × 10⁻⁴) | measured from their moons' motion |
| Any other giant | **resonance locking**: each moon drifts on a time `B` times the system's age, so `T = L / (2 B · age)`. `B` = 2, one editable row | by hand Jupiter gives 0.8 and Saturn 1.7 at their heated moons. One number, no per-planet guess. A law in the fudge register until theory settles |
| Rocky host, no ocean | its own Im k2 from the melt design's response block, at the moon's frequency | the same block the moon uses |
| Rocky host with oceans | a table: Earth today 0.025, long-run mean about a third of that | Earth's ocean basins are near resonance now. With today's value held fixed the Moon would have been at Earth's surface 1.5 Gyr ago, which is wrong. A cited table is Alex's ruling for hard problems; ocean tides from basin shape are far out of reach |
| A star hosting a planet | not in this note | the star's push is negligible for planets; see 3.7 |

### 3.7 Planets in step round a star

Formation already keeps planets' resonant chains. The same rounding-off (3.2) and the same response apply, and a chain forces a small eccentricity on its members, so close-in chains (the TRAPPIST-1 kind) are tidally heated. The difference: nothing feeds them. The heat comes out of the orbits, which slowly spread apart (Lithwick and Wu 2012; Batygin and Morbidelli 2013). That needs the forced eccentricity as a function of how far the pair has spread, which I cannot write from memory. It is package HF6.5, specified after the sources are read. Until then a planet's eccentricity from formation stands, with 3.2's decay applied where nothing forces it.

### 3.8 A moon that falls in

Asked by Alex on acceptance: would a moon hitting its planet show on the timeline? Yes. It is one of the march's marks, and the physics is already in 3.1.

A moon inside the distance where its orbit matches the host's spin, or orbiting backwards, is pulled inward (Phobos, Triton). What ends it is read off two distances:

- **The Roche limit lies outside the host's surface** (the usual case for a rubble or ice moon): the moon is pulled apart there and becomes a ring, at a date. The ring's inner part then rains down over a time the ring model gives.
- **The moon holds together down to the surface** (a small moon held by its own strength, or a dense moon of a low-density host, where the limit lies inside the host): it reaches the surface whole. That is an impact at close to the orbital speed there, shallow and along the equator.

Either way the march publishes a mark with its date and its `why` from the numbers: the moon's mass, its speed, the energy, and which of the two it was. A whole-moon fall is handed to the impact machinery as one more impactor (crater or basin, the heat it leaves in the host's thermal line, what it does to air and sea), with no special case. Onto a giant it is an entry into the air and a mark, and no crater.

How it reaches the timeline: the mark is published through the time design's `happenings` op (its package TP6) with a rank from the sifter, so the timeline view, "what you passed" on a jump and the Run desk's "what is due" all get it from the one list. A moon falling onto a lived-on world ranks at the top. Until TP6 is on main the mark sits in the system's kept table and in `tidal_history`.

Honest limits:

- For a real system the backward sum cannot see a moon that is no longer there, so past falls appear only in generated systems. Future ones appear for both: Phobos in some tens of millions of years (from memory, **unverified**), Triton far later.
- Whether a moon breaks up or holds together is a stand-in (density and size against a strength figure on the sheet), in the fudge register until the sources are read.
- Collisions while worlds are forming (the one that made the Moon) are formation's events, already on the same list, and not this.

Package: HF6.3 for real systems' futures, HF6.4 for generated systems.

---

## 4. Time, and which way the sum runs

Per `SSE4-time-scales-design.md`.

- **A lone moon's distance and eccentricity are lines** (closed forms, 3.1 and 3.2).
- **A system of moons is a march kept as a line.** State: each moon's `a` and `e`, which sets are locked, the host's spin. Anchor: the system's birth. Grid: geometric in time from the first million years (the drift is fast when moons are close), a function of the host and its moons alone. Captures and passages are found by bisection between grid points and are the march's **marks**. The whole state is kept at each point.
- **Own time:** the fastest moon's drift time, `L / 2T`.
- **Coupled to the thermal line by the window rule.** A moon's Im k2 changes its rounding time; the orbit changes the moon's heat. Where the two times are comparable they are stepped together; where the orbit is far slower (the usual case) the thermal line simply reads the heat line.

**Two directions, one mechanism.**

| World | Anchor | The sum runs | What is output |
|---|---|---|---|
| Generated | formation's birth distances and eccentricities | forward from birth | today's orbits, who is in step, the heat line, the events |
| Real or authored (today's orbit is the fact) | today's measured orbit | backward from today, marked "reconstructed" | the past: where each moon was, when sets locked, past heat spans |

The backward sum fills `Body.tidal_history` with worked-out spans where today only Ganymede has an authored one. It cannot see a resonance that has since broken; an authored span still wins where a pack gives one.

**Marks**, each with a `why` from its numbers: caught in step; passed through a resonance (heat pulse); a set breaks; a moon crosses the Roche limit and becomes a ring; a moon falls onto its host (section 3.8); a moon is lost past the edge of the host's hold; the host's day reaches a round length; a moon's orbit rounds off ("the tides stopped").


### 4.1 One leg or many: the same answer

The time design's finding 4 is that ageing a system in two legs differs today from ageing it in one. The moon march must not add a second case. It meets that design's march rule point by point, and each point is a build rule for HF6.3:

1. **Fixed anchor.** A generated system's anchor is its birth. A real or authored one's is the date its orbit was measured at (the pack's epoch: J2000 for Sol). Neither is the date asked from or to, and never the campaign's "now". An edit to a moon or its host writes a checkpoint in the chronicle, and the march restarts from that.
2. **The grid belongs to the subject.** Grid dates are counted from the system's birth in both directions, so a real system's backward sum and a generated system's forward sum land on the same kind of grid. The anchor date is one extra kept point on it.
3. **The whole state is kept at every point:** each moon's distance, eccentricity and the angle it has turned through since the anchor, which sets are in step, the host's spin, and the date. A capture or a passage found by bisection is kept as a point of its own, holding the state just after it.
4. **A read between points is a stated rule:** the closed forms of 3.1 and 3.2 from the kept point next nearer the anchor. Nothing is ever stepped from the date asked from to the date asked to.
5. **A window with the thermal line** (where the two times are comparable) runs from kept point to kept point on a step set by the state alone.
6. **A real system's future is read from the same table as its past.** The reconstructed past is never marched forward again to reach today, so today is always the measured orbit, exactly.

With these, a read at a date and a walk there in any number of legs give the same bits. That is HF6.3's first red-first test, sharing the time design's two-legs harness (its package TP5).

One ask of the time design: its march rule names two anchors, birth or a checkpoint. A measured orbit's own date is a third, used here and already by any real body. *Recommend the time design names it (choice D6).*

### 4.2 Living with measured motion

"Make eclipses happen on time" (RW17 part 1, PR 649, open at the time of writing; read from its description, package log and tables, not its whole diff) gives a catalogued body a **motion model**: inside the model's span the body is where a published series says, and outside it a plain orbit fitted at the span's edge takes over. Sol's planets name VSOP87A and the Moon ELP/MPP02, both spanning about 4,000 years either side of J2000. Each placed time says `measured` or `plausible`.

The two answer different questions on different spans of time. The motion model says where a body is, to seconds, over thousands of years. The moon march says how an orbit's size and shape change over millions and billions of years. The rule that keeps them apart:

- **Inside a motion model's span, the series is the whole answer and the march moves nothing.** The Moon's measured drift is already inside ELP/MPP02 (its slow-down is a term of the series), and Earth's slowing spin is already in that work's Delta T. Adding the march's drift on top would count the tide twice. Over the whole span the Moon moves out about 150 m, far below anything the march resolves, so nothing is lost.
- **Inside the span the march is still published, beside the measurement, as a check:** "measured 3.8 cm a year; worked out 3.7". The measurement is the fact. The same holds for the length of the day.
- **Outside the span, the edge fit stays the base and the march scales it.** A body with a motion model takes its plain orbit from the fit at the span's edge, exactly as PR 649 does. The march then supplies only ratios: distance and eccentricity are the fitted values times the march's value at the date over its value at the edge, and the angle along the orbit advances by the march's kept angle since the edge. At the edge every ratio is 1, so the handover PR 649 already makes is unchanged to the bit, and where there is no march (a body with no host tide worth the name) the behaviour is PR 649's own.
- **The march's anchor for such a body is the series' mean orbit at J2000,** not the edge fit. Only ratios cross from the march to the placed orbit, so the two never have to agree on a value, only on a shape.
- **Which way Earth faces** follows the same split: PR 649's measured rotation inside its span; outside it, the host's day from the march, joined at the edge.
- **Labels.** A time placed outside the span stays `plausible`, with its source now naming the tide march. No third word is added (choice D7).
- **Bodies with no motion model** (every generated moon; Jupiter's and Saturn's moons, which PR 649 leaves on plain orbits) take the march's line directly, anchored at their plain orbit. If a later package gives them a series, the rule above applies with no change here. Finding which moons are in step reads period ratios within a tolerance, so it does not need the series.

By package: HF6.1 and HF6.2 do not touch where anything is placed, so they cannot meet the slot at all. HF6.3 is the first to change `orbits_at`, and only outside a span. It needs PR 649 on main first, and three absolute tests seen red: every eclipse test of PR 649 passes unchanged; the placed position across a span's edge jumps by no more than it does on PR 649 alone; a date outside the span read in one leg equals the same date reached in two.

A further check this opens, to be read at source after 1 Dec: the tide's share of the lengthening day (about 2.3 ms a century, from memory, **unverified**) against the march's own figure for Earth.

---

## 5. Hand checks

Measured inputs from memory, unverified. None was adjusted.

| Check | By hand | Measured |
|---|---|---|
| Moon's drift today, Earth's k2/Q 0.025 | 3.7 cm/yr | 3.8 cm/yr |
| Io's rounding time alone, Im k2 0.015 | 1.5 × 10⁵ yr | (must be far less than its age: yes) |
| The Moon's rounding time, k2 0.024, Q 38 | 12 Gyr | (must exceed its age: yes) |
| Io, Europa, Ganymede: heat to shed, Jupiter's k2/Q 1.1 × 10⁻⁵ | 6.5 × 10¹³ W | Io 0.9 to 1.0 × 10¹⁴ W |
| The same, as drift of the set | once in 6 Gyr | Io measured moving slightly inward: it is not settled |
| Enceladus and Dione, Saturn's Q 18,000 | 1.0 GW | published sum 1.1 GW (Meyer and Wisdom 2007) |
| Enceladus and Dione, Saturn's k2/Q 1.6 × 10⁻⁴ | 8 GW | 10 to 16 GW |
| Drift time over age, Io | 0.8 | resonance locking predicts about 1 to 3 |
| Drift time over age, Enceladus | 1.7 | the same |
| Io's eccentricity if the interior is the melt design's (Im k2 about 10⁻³) | about 0.016 | 0.0041 |

**Expected accuracy.** Heat of a settled set: within a factor of two. Drift rates where the host's response is measured: within a tenth. Eccentricity of a heated moon: within a factor of a few, since it carries all the interior's uncertainty. Dates of capture: a factor of two at best, since they hang on the host's response over billions of years, which is the least known number in the whole sum. The Moon's past distance: only as good as the ocean table.

---

## 6. What changes for whom

- **The thermal line (melt design)** takes a moon's tidal heat from the balance where a set is in step, from 3.2's decay where it is not, and from a measurement where one exists. Its own Im k2 feeds back as the eccentricity and the rounding time.
- **Formation** stops handing moons a lasting eccentricity. The moon march replaces F7's moon part. Generated giants' inner moons will often be in step (as Jupiter's, Saturn's and Uranus's are, or were). **This changes generated systems: moon distances, who is heated.**
- **Rings.** A moon driven inside the Roche limit is the physical source of a massive ring, at a date. Ring and belt editing's "raising a ring's mass offers breaking up a moon first" gets a real candidate to offer.
- **The planet's day.** Spin from formation, slowed by its moons, as a line. Climate and weather read spin already.
- **Locking.** `lock_time_gyr` reads the body's own response in place of one Q for all.
- **"What is interesting here".** A heated moon and why; a moon about to become a ring; a resonance that broke; a day that was once six hours long.
- **Players and GMs** see the same orbits as now at the campaign's date. Nothing on screen moves faster.
- **Measured motion (RW17).** Untouched inside a motion model's span; outside it the edge fit is scaled by the march (section 4.2).

---

## 7. Numbers and blocks

Sheet rows (all `unverified`):

| Row | On | Value | Source |
|---|---|---|---|
| `tides.k2_over_q` | Jupiter, Saturn (reference objects) | 1.1 × 10⁻⁵, 1.6 × 10⁻⁴ | Lainey et al. 2009, 2017 (also in the melt design) |
| `tides.k2_over_q` | Earth | 0.025 today; long-run mean 0.008 | Williams and Boggs 2016; Farhat et al. 2022 |
| `tides.k2_over_q` | Uranus, Neptune (for checks) | about 10⁻⁵ to 10⁻⁶ | Tittemore and Wisdom 1990; Zhang and Hamilton 2008 |
| `tides.locking_time_over_age` | convention for giants | 2 | Fuller et al. 2016; Lainey et al. 2020 |
| `orbit.eccentricity`, `orbit.semi_major_axis` of the heated moons | exist in `base-sol` | | |

Blocks, family `sse.tides` (formula blocks unless said):

| Block | Does |
|---|---|
| `sse.tides.host_torque` | 3.1, first line |
| `sse.tides.orbit_drift` | 3.1, the drift and the closed form |
| `sse.tides.eccentricity_damping` | 3.2, with the host's small raising term |
| `sse.tides.locked_set_heat` | 3.4 |
| `sse.tides.balance_eccentricity` | 3.5 |
| `sse.tides.resonance_locking_torque` | 3.6, second row |
| `sse.tides.host_spin` | spin from the total angular momentum |
| `sse.tides.moon_system` (Big Block) | the march of section 4; calls the blocks above. Its view: distance against time for every moon of a host, with the captures marked |
| `sse.tides.host_power_ceiling` (melt design) | becomes the one-moon case of `locked_set_heat` |

Fudge register: capture certain when slow and converging; passage kick 0.01; equal forced eccentricities in a generated set; the 0.1 limit on a held eccentricity; `B` = 2 for giants; Earth's ocean table; heating cycles not modelled, only their mean.

---

## 8. Packages

Owner: proposed as its own builder thread ("Build moon orbits and tides"), since it works in formation and orbits where the heat flow thread works in the interior. Each leaves main green.

| Package | What | Needs | Golden | Size |
|---|---|---|---|---|
| **HF6.1** the balance for real moons | The blocks of section 7 except the march; resonant sets read off period ratios; the balance published beside the measured heat for Io, Europa, Ganymede, Enceladus, Dione; the Moon's drift and its rounding time as facts. Nothing reads them for physics yet | the melt design's HF4c.1 for Im k2 (or the shared constant until then) | area only | S |
| **HF6.2** lone moons round off | 3.2 applied: a moon with no partner has its eccentricity and heat decayed to its age. Sol's lone moons barely move (their measured `e` is the fact); fixtures with authored heat do not move | HF6.1 | core, one slot | S |
| **HF6.3** the moon march | The Big Block of section 4 for real systems, run backward: reconstructed distances, capture dates, `tidal_history` spans, the host's day as a line, the marks | HF6.2; TP1 for dates; TP5's two-legs harness; TP6 for marks on the timeline (kept in the table until then); PR 649 on main (section 4.2) | core, one slot | M |
| **HF6.4** generated moons | The march run forward from formation: captures, heated sets, moons lost to rings; formation's moon eccentricities become birth values; generated moons handed to the surface code | HF6.3; Alex's yes to question 1 | **core, one slot; changes generated systems** | M |
| **HF6.5** planets in step | 3.7, after the sources are read | HF6.2; 1 Dec | core | M |
| **HF6.6** ocean tides worked out | A sea's tide from the world's own depth map and liquid, in place of the table (section 10A). **Later; not queued** | depth maps and liquid types | | |

Red-first tests: a lone moon's heat falls to nothing over many rounding times (fails today); energy and angular momentum of host plus moons conserved over a march to 1 part in 10⁶ (the absolute check); the march read at a date equals stepped there in two legs, to the bit (section 4.1); PR 649's eclipse tests unchanged and no added jump at a span's edge (section 4.2); the Moon's drift within a tenth of 3.8 cm/yr; the three-moon sum of section 5 reproduced.

---

## 9. What could be wrong

- **The host's response over billions of years is the weakest number.** Resonance locking is a young idea with good support for Saturn and less for Jupiter. If it is wrong, capture dates move a lot and today's heat only a little.
- **Capture and passage** are stand-ins until Murray and Dermott's chapter is read.
- **Real moons are not all settled.** Io is measured to be shedding more than Jupiter feeds it. The balance is a mean; the measurement stays the fact.
- **The backward sum cannot find broken resonances.** Reconstructed histories will miss past heat pulses that authored spans now give.
- **Earth and the Moon** rest on a table, honestly labelled.
- **Enceladus's 10 to 16 GW** is reached only with Saturn's newer measured value, and the measurement of Enceladus's heat itself is disputed between about 5 and 16 GW.

---

## 10. Questions for Alex

1. **May generated moons be changed by this?** Today a giant's moons sit at random spacings with a random stretch that lasts for ever. With HF6.4 they would move, often catch each other in step, sometimes be lost to rings, and only those held in step would stay hot. Existing generated systems would change (you have said existing universes can be thrown away). *Recommend yes.*

2. **For giant planets, one law in place of a dissipation number per planet?** "Each moon drifts on a time about twice the system's age" matches Jupiter and Saturn by hand and needs nothing typed per planet. It is a law with good but not settled support; the measured values still win for Jupiter and Saturn. *Recommend yes, registered as a stand-in for the theory.*

3. **Earth's tides from a cited table?** Ocean tides depend on the shape of the ocean basins, which is out of reach. A table (today's value and the long-run mean) keeps the Moon's past honest. This is your "worked examples and tables" ruling applied. *Recommend yes.*

4. **Its own builder thread?** The work is in formation and orbits; the heat flow thread is deep in interiors. *Recommend yes: "Build moon orbits and tides", opened when the melt design's HF4c.1 is on main.*

## 10A. Alex's answers, 26 Nov 2026 (23:33 UTC)

"Yes to all, make sense!" All four recommendations of section 10 are rulings:

1. Generated moons may be changed by this. Existing generated systems change.
2. One drift law for giant planets, registered as a stand-in for the theory; measured values still win for Jupiter and Saturn.
3. Earth's tides come from a cited table for now.
4. Its own builder thread, "Build moon orbits and tides", opened when the melt design's HF4c.1 is on main.

His two notes:

- **"Would moon impacts with planets appear on the timeline view?"** Yes: section 3.8, added in answer.
- **Ocean tides worked out, later.** "We will have depth maps of the whole planet and liquid types defined - so potentially we can get that at some point. It's a nice to have.. but sounds a bit early here." So the table of answer 3 is the stand-in, in the fudge register, and a worked-out ocean tide from a world's own depth map and liquid is a later package (named HF6.6 here so it is not lost). It is not queued and nothing waits on it. When built it would serve any world with a sea, not only Earth.

## 11. Routine choices, for the coordinator

- **D1** Package names HF6.1 to HF6.5; three core-lane turns before HF6.5. *As written.*
- **D2** HF6.1 may start before Alex answers: it publishes facts beside measurements and changes no physics. *Recommend yes.*
- **D3** F7's moon part is taken to be this; the formation design's F7 row gains a pointer. *Recommend yes.*
- **D4** The moon-distance chart (the march's view) goes with the thermal-line chart of the melt design's C6. *Recommend yes.*
- **D5** The citations thread reads section 12 on 1 Dec with the melt design's list. *Recommend yes.*
- **D6** The time design's march rule names a third fixed anchor, a measured orbit's own date (section 4.1). An ask of that design's owner. *Recommend yes.*
- **D7** A time placed outside a motion model's span stays `plausible`, its source naming the tide march; no third word beside `measured` and `plausible` (section 4.2). *Recommend yes.*

---

## 12. Sources, all unverified until 1 Dec 2026

| Source | Used for |
|---|---|
| Batygin, K. and Morbidelli, A. 2013, AJ 145, 1 | planets in step spreading by tides |
| Dermott, S. F., Malhotra, R. and Murray, C. D. 1988, Icarus 76, 295 | capture and passage through resonance |
| Farhat, M. et al. 2022, A&A 665, L1 | Earth's ocean tides through time |
| Fuller, J., Luan, J. and Quataert, E. 2016, MNRAS 458, 3867 | resonance locking |
| Howett, C. J. A. et al. 2011, JGR 116, E03003 | Enceladus's heat |
| Lainey, V. et al. 2009, Nature 459, 957 | Jupiter's k2/Q, Io's heat and motion |
| Lainey, V. et al. 2017, Icarus 281, 286 | Saturn's k2/Q |
| Lainey, V. et al. 2020, Nature Astronomy 4, 1053 | Titan's fast drift |
| Lithwick, Y. and Wu, Y. 2012, ApJ 756, L11 | planets in step spreading by tides |
| Meyer, J. and Wisdom, J. 2007, Icarus 188, 535 | the balance for Enceladus (1.1 GW) |
| Murray, C. D. and Dermott, S. F. 1999, Solar System Dynamics (already cited) | tides, resonance, forced eccentricity |
| Ojakangas, G. W. and Stevenson, D. J. 1986, Icarus 66, 341 | heating cycles of Io |
| Showman, A. P. and Malhotra, R. 1997, Icarus 127, 93 (already cited) | Ganymede's past resonance |
| Tittemore, W. C. and Wisdom, J. 1990, Icarus 85, 394 | Uranus's moons and its Q |
| Williams, J. G. and Boggs, D. H. 2016, Celest. Mech. Dyn. Astron. 126, 89 | the Moon's drift, Earth's tidal response |
| Zhang, K. and Hamilton, D. P. 2008, Icarus 193, 267 | Neptune's Q |
