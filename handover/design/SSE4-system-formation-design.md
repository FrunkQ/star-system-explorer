# SSE4 system formation design: how a star system is made, and how it lives

Written 18 Nov by the design thread at Alex's request. Authority for star system generation in SSE4: what System Lab becomes inside the physics suite. Read with the handoff (H) sections 5 and 6, the galaxy design (section 8, which hands a newborn system its ingredients), the classification design (reverse placement), the surface design and the life design (which take over each world once it exists).

Sources: Alex's System Lab plan (one seed gives a deterministic formation history with bodies, accretion events, material bands, provenance, uncertainty and reproducible time slices; must cope with passing stars, galactic motion and Oort cloud disturbance, REBOUND or REBOUNDx, detailed stellar evolution with mass transfer and mergers, and long-term geology, climate and atmosphere); SSE 3's `v4-scope.md` ("generation has a TIMELINE", "build one system, parameterised, not three", "complexity out of layered simplicity", precomputed snapshots that the scrubber reads); current planet formation science.

---

## 1. What was already designed, and what this adds

Already in the design: a system inherits its ingredients from its patch of the galaxy (galaxy design 8.4); a resolved thing gets its whole life line drawn once, and any date is a read of it (H6); quiet stretches are stepped analytically and interesting moments open a detailed window (H6, epoch stepping); stellar encounters are drawn as dated events from the patch (galaxy design 8.3); heavy blocks are swappable and ship results as data first (H11, H14); surfaces, climate and life have their own designs.

What was thin: the middle. The build plan had one line, "system formation from a patch envelope". This document fills it in: the disc, material bands, growth, giants, migration, the violent phase, moons, belts and clouds, each planet's early life, and the long-term and outside influences, as **nine stages with one timeline**. It also adds the Formation screen.

## 2. Principles

1. **Generation is a timeline.** A system is not rolled, it happens. Every body has a birth, a growth record, events and an end. The time ruler scrubs a stored history; scrubbing never recomputes (SSE 3's own conclusion, kept).
2. **One generator, parameterised.** The same nine stages make a random system, a system grown around a GM's pinned world, a Traveller UWP system (H7A), an imported V3 system and the Solar System. They differ only in what is pinned.
3. **Layered simplicity, not simulation.** Each stage is a small flow of readable rules with published relations behind them. Where real dynamics are chaotic, the outcome is a keyed draw from distributions calibrated against detailed studies, and labelled as such. A detailed block can replace any stage later without changing anything around it.
4. **Budgets are conserved.** The disc's mass, each element group and angular momentum are budgets. Everything that forms claims from them; nothing is created from nowhere (H6 no double counting).
5. **Provenance on everything.** Each body carries a ledger: which material band each part of it came from, and when. That is where "why does this world have so much water" gets its answer.
6. **Uncertainty is shown.** Every value says whether it was calculated, looked up in a table, drawn from a calibrated distribution, or authored, and how wide the spread was.
7. **Clouds until looked at.** An unvisited system stays an envelope (counts and distributions). Looking resolves the whole history once, deterministically, from the seed and the address.

## 3. The nine stages

Times are typical for a Sun-like star and scale with the star.

| # | Stage | When | What happens | What it leaves behind |
|---|---|---|---|---|
| 1 | **Inheritance** | before 0 | The patch hands over a cloud core: mass, spin, element groups (so rock chemistry, water and radioactive heating are set by what died there before), short-lived aluminium-26, and the birth cluster's crowding and ultraviolet glare | the system's starting budgets |
| 2 | **Star or stars** | 0 to 0.5 Myr | The core collapses and may split: single, binary or more, with masses and separations. Each star gets its whole life line from tables: brightness and size over time, and how it ends | stars with life lines; for close pairs, dated mass-transfer and merger events |
| 3 | **Disc and material bands** | 0 to about 5 Myr | A disc of gas and dust with a size, mass and lifetime (cut short by cluster glare or a companion star). It cools over time, so the ice lines for water, carbon dioxide and carbon monoxide sweep inward. Between the lines lie **material bands**, each with its own mix of metal, rock, carbon, ice and gas | bands that change with time: the system's larder, and the root of all provenance |
| 4 | **Growth** | 0.1 to 5 Myr | Solids pile up at band edges and pressure bumps into rings of planetesimals, then embryos. Embryos feed on pebbles drifting inward until they starve their inner neighbours | embryos with growth records; rings where growth stalled |
| 5 | **Giants and migration** | 1 to 5 Myr | A core that gets heavy enough before the gas leaves pulls in gas and becomes a giant, opens a gap, and drifts through the disc, shepherding or scattering smaller bodies and often ending in resonant chains. The disc's end is the deadline | giants, sub-Neptunes, chains; many systems get no giant at all, more so at low metal content |
| 6 | **The violent phase** | 5 to 300 Myr | With the gas gone, crowded systems rearrange: chains break, orbits cross, embryos collide in giant impacts, some bodies are thrown out as rogue planets. Impacts make large moons, strip mantles, set tilts and spins. Late arrivals from outer bands deliver water and carbon | final planets with impact histories, tilts, spins; rogue planets as real objects; delivered water with provenance |
| 7 | **Leftovers** | 5 Myr onward | Belts where a giant stalled growth, an outer belt, a scattered disc, an Oort cloud built from what the giants flung outward and the cluster's pull kept, Trojans, captured irregular moons, regular moons from discs around giants, rings | reservoirs as clouds that resolve into bodies when looked at |
| 8 | **Each world's early life** | 0 to 1 Gyr | Magma ocean, separation into core and mantle, loss or keeping of the first hydrogen air (which makes the gap between large rocky worlds and small gas-rich ones), outgassed second air, oceans condensing, first crust | a starting state handed to the surface, climate and life designs |
| 9 | **Long life and the outside** | 0.3 Gyr to the end | Slow orbital change, tides (locking, moons receding or spiralling in), the star brightening, climate and air evolving, geology, life. From outside: passing stars and the galactic tide shake the Oort cloud into comet showers, spiral arm crossings, nearby supernovae. Finally the star swells, sheds mass, orbits widen, inner worlds are swallowed, and the remnant cools | a full future as dated events, to the system's end |

Each stage is a flow in the flow editor, built from atoms with the three explain lenses (H6 EXPLAIN). Each writes dated events into the chronicle, so the story sifter can find "the impact that made the moon" or "the comet shower when the red star passed".

## 3A. Upheavals: the "good story" after the planets form **[Alex, 18 Nov]**

Alex's point: Accrete works hard but stops when the planets exist. It never made the star, and nothing happens afterwards, so there is no story. Stages 1 and 2 put the star in front. This section is the part after: a deterministic **upheaval system** in the spirit of the Nice model, producing the bagatelle that made our Moon, without running an orbit simulation by default.

**Two sources of trouble.**
- **From outside**: passing stars, nearby supernovae, the galactic tide, arm crossings. These are not drawn at random at all: they are read from SSE4's own star map and its motions (galaxy design 8.3 and 9), so the red dwarf that shook this system's Oort cloud is a real, visitable star.
- **From inside**: the system's own instability. This is the hard part and is designed here.

**How an inside upheaval is worked out, in four steps.**

1. **Is the system primed?** After the gas leaves, the kernel measures how close the system is to trouble, using published stability rules: how tightly neighbours are spaced for their masses, whether giants sit in a resonant chain, how much orbital "slack" the system holds, and how massive the leftover outer belt is (the Nice trigger: giants slowly trading places with belt material until two of them cross a resonance). The result is a hazard: a chance per million years that the arrangement breaks.
2. **When does it break?** One keyed draw against that hazard gives the date, or never. Calm systems stay calm; packed ones break early; a giant chain beside a heavy belt breaks late, which is what gives a system a late bombardment.
3. **The cascade.** A break starts a chain of typed events, each drawn from odds calibrated on published orbit-integration studies and each handing its consequences to the next. The system's energy and angular momentum are budgets the whole chain must respect, so the outcome is always a system that could exist.

| Event | Typical consequences it passes on |
|---|---|
| Giants cross a resonance, orbits stretch | belts stirred and thinned; inner worlds' orbits pumped |
| Giant scatters giant | one moves inward on a stretched orbit, one outward or **ejected as a rogue planet** |
| Giant migrates through a belt | **bombardment spike** on every world, recorded as craters (surface design), air and water delivered or stripped, a catastrophe for any life (life design); Oort cloud filled; Trojans and irregular moons captured |
| Inner orbits cross | **giant impact**, outcome by speed and angle from scaling laws: merge, graze and merge with a **large moon**, hit and run, mantle stripped to leave an iron-rich world, or both shattered into a belt |
| World thrown inward or outward | swallowed by the star, parked on a wide cold orbit, or lost |
| Tilt and spin knocked | seasons, day length and tidal futures change |
| Moon system disturbed | moons lost, swapped or collided into rings |

4. **Settling.** Stretched orbits calm down over a stated time through tides and leftover debris, and the system reaches its lasting arrangement. Every event is a dated chronicle entry with its cause, so each asks "why" all the way back to the trigger.

**Why this is deterministic and still surprising.** Every draw is keyed by the seed and the event's address, so the same system always has the same upheaval, on every device. But the cascade branches on its own results, so systems differ wildly: most have a quiet rearrangement, some lose a giant, a few end with a hot giant on a stretched orbit and nothing else.

**Later upheavals reuse the same cascade.** A close stellar pass, a companion star's orbit slowly squeezing the planets, mass loss as the star dies, or a GM saying "a rogue planet comes through in year 3000" all feed step 3 with a different trigger. One mechanism, many causes.

**For the GM**: upheavals appear as marked spans on the formation chart and the time ruler. Each can be opened, re-timed, softened, removed or replaced (as a branch), and a GM can add one by hand. The story sifter ranks them, since they are the most tellable things in a system's past.

**The optional heavy block.** A Detailed block replaces step 3 with real orbit integration for that window (section 4), writing the same event types, so nothing downstream changes. The Standard cascade is tested against it by bounded agreement: the mix of outcomes over many systems must match, not each individual system.

## 4. What is calculated, what is drawn, and the fidelity ladder

Every stage declares a block at one of three levels. The level in use is always visible, with its cost class (H14).

| Level | How | Runs on | Used for |
|---|---|---|---|
| **Sketch** | closed formulas and occurrence statistics | anything, instantly | envelopes of unvisited systems, galaxy-scale surveys, weak devices |
| **Standard** (default when a system is resolved) | band-by-band growth and migration rules stepped in time; calibrated draws for chaotic outcomes; tabulated stellar tracks | a laptop in about a second, a phone in a few **[estimate]** | every system a GM opens |
| **Detailed** (plug-in blocks, later) | direct orbit integration in windows, finer stellar and binary tables, coupled climate | capable devices, or results fetched as data | proving the Standard level, and GMs who want it for one special system |

- **Classic Accrete block.** SSE 3's Accrete code (now standing alone as System Lab) is ported as an alternative Standard block for stages 4 and 5. It keeps parity for V3 users and gives an honest comparison against the modern pebble-and-migration block. Both write the same outputs.
- **Bounded agreement** (H12): each Standard block is tested against its Detailed counterpart or published populations on named measures (how often giants form against metal content and star mass, spacing of neighbours, the radius gap, moon-forming impact rates).
- **Orbit integration [decided in outline, Alex 18 Nov: "rewrite REBOUND in Rust"].** Direct integration is needed only in windows (the violent phase of one system, a close stellar pass). SSE4 carries its **own integrator, written fresh in Rust in the kernel** from the published methods: a Wisdom-Holman symplectic scheme for calm stretches (the method behind REBOUND's WHFast), a high-order adaptive scheme for close encounters (the method behind IAS15), a hybrid switch between them (as in MERCURIUS), plus the extra forces REBOUNDx is used for (tides, general-relativity correction, mass loss, a passing star's pull). Fixed step rules and pinned maths make it bit-identical on every device. **It must be written from the papers, not ported from REBOUND's source**: REBOUND is GPL, and a line-by-line translation would carry that licence into an Apache-2.0 codebase. REBOUND and REBOUNDx are used as the **reference it is tested against**, outside the shipped code. Credit the papers in NOTICE and the explain text.
- **Stars.** Life lines come from tabulated tracks by mass and metal content (results-as-data). Close binaries use rule tables for mass transfer, common envelopes and mergers, producing dated events. Detailed stellar evolution is a later plug-in that replaces the tables.

## 5. Records the generator keeps

```
FormationHistory { system address, seed, manifest ref,
  budgets:  { disc gas, solids by element group, angular momentum }      with what claimed each part
  bands:    [{ id, inner, outer (both functions of time), mix by group, temperature, gas present until }]
  bodies:   [{ address, born: time and band,
               growth: [{ time, mass, orbit, source band, amount }],     the provenance ledger
               events: [chronicle refs: gap opened, migrated, impact, capture, ejected, engulfed ...],
               state at each stored slice }]
  slices:   [times]             stored snapshots; the ruler interpolates between them, never recomputes
  evidence: per value: calculated | tabulated | drawn (with spread) | authored | reconstructed
  level:    per stage: sketch | standard | detailed, and which block }
```

**Time slices** are stored densely where things change fast (the first 300 million years) and sparsely afterwards. Any slice reproduces from the manifest on any device.

## 6. Pins, edits and reverse requests

- **Pin anything, at any time in the history.** A pinned star, planet, orbit or event is honoured, and the stages are solved around it. The search is the same deterministic search over causes used for reverse classification (classification design, section 7): candidate histories are run and the best fit kept.
- **Edits act from their date forward.** Remove the giant at 2 million years and everything after is re-derived on a branch, the original kept (H6 branches).
- **"Put a temperate ocean world here"** (class atlas) is a pin on the end state; the generator finds a history that produces it, or says why none exists.
- **Imported systems** (V3, UWP, catalogue stars and exoplanets) arrive with no history. The generator fits a plausible one and labels it **reconstructed**, so the time ruler works on them too. Ageing the Solar System, the top user wish, is this: real bodies as pins, reconstructed past, computed future.
- Impossible pins are kept and marked, as everywhere.

## 7. The Formation screen

Mockup: "System formation". A GM workspace screen in the chart-room style; it is what System Lab's interface becomes.

- **The formation chart** is the centrepiece: distance from the star across, time down the page, both on stretched scales. Material bands are shaded columns whose edges (the ice lines) lean as the disc cools. Each body is an inked track that thickens as it grows and bends as it migrates; tracks merge at impacts and run off the edge at ejections. The gas disc is a tint that ends when the gas leaves. Belts are hatched strips. Events are diamonds. A horizontal rule is the time slice, tied to the time ruler: drag it and every other view shows the system at that date.
- **Left**: what was inherited from the patch, the seed, and the level in use for each stage with the device edge.
- **Right**: the selected body's story: where its material came from (a provenance bar by band), its events in order, and how sure each number is.
- **Below**: the nine stages as a strip to jump along; "how this was worked out" (calculated, tabulated, drawn); and actions: pin, branch from here, ask why, compare blocks (Classic Accrete against the modern block), open in the flow editor, ask Orac.
- It fits with the rest: the system chart shows *now*; this chart shows *how it got here and where it is going*. One click moves between them, holding the selected body.

**On each desk** (three desks note, section 13A; built as "How it formed", `status/formation-screen.md`):

| Desk | What the screen shows |
|---|---|
| Explore | The whole reading half: chart, stages, slice, Inherited, the selected world's story. Moving the slice moves the ruler to look; nothing is recorded |
| Build | The same, and later the actions that change the story (pin, branch from here), which write a branch |
| Run | Not on the rail (deep time is not Run's) |

## 8. Stages for the build (track F, replacing package P2)

| Stage | Contents | Needs | Evidence |
|---|---|---|---|
| F0 | Inheritance from the patch envelope; stars, multiplicity, tabulated life lines; FormationHistory record and time slices | P1, K7, G0 | same seed gives the same history on every device; star properties match the tables' sources; a slice reproduces from its manifest |
| F1 | Disc and material bands with moving ice lines; budgets | F0 | band edges match cited disc models at three ages; budgets sum exactly |
| F2 | Growth and giants, Classic Accrete block ported from SSE 3 and System Lab | F1, K0 | reproduces System Lab outputs for a fixture set of seeds, or each difference is explained; provenance ledger sums to each body's mass |
| F3 | Growth and giants, modern block: planetesimal rings, pebble growth, gas capture, gaps, migration, chains | F1 | over 10,000 systems: giant frequency rises with metal content and star mass, the radius gap appears, compact chains appear at the published rates within stated tolerance |
| F4 | The violent phase and the upheaval system (3A): priming measure, break date, event cascade with conserved budgets, impact outcomes by scaling law, ejections, bombardment spikes handed to surfaces and life, captures, settling; reusable for later triggers | F3 | neighbour spacing and eccentricity distributions match published populations; rogue planets are addressable objects; water provenance sums |
| F5 | Leftovers: belts, outer belt, scattered disc, Oort cloud, Trojans, moons, rings, as clouds that resolve | F4 | reservoir masses conserve the budgets; a resolved belt object is the same on every device |
| F6 | Each world's early life and hand-off to surface (S), climate and life (L) | F4, P1 | hand-off values are the ones those suites read; no quantity has two publishers |
| F7 | Long life: slow orbital change, tides, the ageing star and its end, binary mass transfer and mergers by rule table | F0, T1 | Solar System fixture: reconstructed past and computed future agree with published accounts at named dates |
| F8 | The outside: stellar passes, galactic tide, Oort showers, arm crossings, nearby supernovae; the kernel's own integrator for windows, tested against REBOUND | F5, G6, T1 | integrator agrees with REBOUND on reference cases within stated error and is bit-identical across devices; shower events follow from encounter events |
| F9 | Pins, edits from a date, reconstruction for imported systems, reverse requests | F4, C3 | a pinned world survives regeneration; an imported V3 system gains a labelled reconstructed history; a class request round-trips |
| F10 | The Formation screen | F1, U3 | reproduces the mockup; dragging the slice moves every view; block comparison shows both results side by side |

F0 to F2 and F6 are enough for the "One system" milestone (a real generated system with a timeline, at parity with SSE 3). F3 to F5, F9 and F10 make it better than SSE 3. F7 and F8 belong with time and travel and with the galaxy.
