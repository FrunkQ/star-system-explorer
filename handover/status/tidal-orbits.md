# Status: the orbit side of tidal heat (design for HF6, track HF)

Owner thread: "Design melting inside worlds" (Fable, design only, 26 Nov 2026) for the note. No builder yet; the note proposes its own builder thread, "Build moon orbits and tides" (its question 4).
Design: `design/SSE4-tidal-orbits-design.md`, **PROPOSED 26 Nov 2026, not yet accepted by Alex.** Started at the coordinator's word as a proposal, ahead of Alex's answer to question 3 of `design/SSE4-melt-and-heat-design.md`. Four questions for him are in its section 10; routine choices for the coordinator in its section 11 (D1 to D7 all agreed by the coordinator 26 Nov).

What it is, in one line: a moon's tidal heat is paid for by its host's spin through the orbit, so the orbit is marched with the heat: the host's tide pushes each moon outward, the moon's own tide rounds its orbit off, and only moons held in step with a partner stay stretched and hot.

Honest state, 26 Nov 2026:

- Nothing is built. The checks in the note's section 5 are hand sums, not a model run.
- Every paper figure is from memory and unverified: the Consensus search is out until 1 Dec 2026 (the note's section 12 lists the sources to read). The two central formulas are derived in the note from conservation of energy and angular momentum.
- By hand the balance gives Io, Europa and Ganymede together about 6.5e13 W against about 1e14 W measured for Io, and the Moon's drift 3.7 cm/yr against 3.8 measured. Enceladus comes out near 1 GW with Saturn's older dissipation number and near 8 GW with the newer measured one, against a disputed 5 to 16 GW measured.
- The host's response over billions of years is the weakest number. Capture into step and passage through it are stand-ins until the textbook chapter is read.

| Item | State |
|---|---|
| The note in `handover/design/`, this file, the row in START-HERE | done, 26 Nov |
| Alex's answers to questions 1 to 4 | waiting |
| Coordinator's answers to D1 to D5 | agreed 26 Nov as recommended; core-lane turns go through the coordinator; HF6.1 may start before Alex answers |
| Coordinator's answers to D6, D7 | agreed 26 Nov. D6 (a measured orbit's own date as a third fixed anchor) was put to the time design's thread the same day, to go into `design/SSE4-time-scales-design.md` |
| How it lives with the time design (PR 651) and measured motion (PR 649) | written 26 Nov: the note's sections 4.1 and 4.2. Inside a motion model's span the series is the whole answer; outside it the march scales the edge fit. HF6.3 needs PR 649 on main and the two-legs harness of TP5 |
| Read the sources at source and correct the sheet rows | from 1 Dec 2026, citations thread (D5) |
| A builder thread | not opened; proposed for when the melt design's HF4c.1 is on main |

Packages (no owner yet): HF6.1 the balance for real moons (area golden only; may start before Alex answers, D2), HF6.2 lone moons round off, HF6.3 the moon march for real systems, HF6.4 generated moons (needs Alex's yes to question 1; changes generated systems), HF6.5 planets in step (after 1 Dec). Core golden lane: HF6.2, HF6.3, HF6.4, HF6.5, one turn each.
