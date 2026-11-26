# Status: melting inside worlds (design for HF4b, track HF)

Owner thread: "Design melting inside worlds" (Fable, design only, 26 Nov 2026) for the note. The build belongs to the thread "Fix low heat flow in worlds", which keeps `status/heat-flow.md`; this file only tracks the design and its questions.
Design: `design/SSE4-melt-and-heat-design.md`, **PROPOSED 26 Nov 2026, not yet accepted by Alex.** Four questions for him are in its section 13; seven routine choices for the coordinator in its section 14.

What it is, in one line: one melt mechanism for any solid world (melt share between solidus and liquidus, melt leaving by its own buoyancy and taking heat, water and the heat-making elements with it, rock softened while melt is in it), with the lid as its own kept state, volcanism as the melt that leaves, and tidal heat read from that state each step.

Honest state, 26 Nov 2026:

- Nothing is built. A scratch model (project files `sse4/melt-proto/`, a port of `core_history` at PR 660's head plus this mechanism) was run on Mars, the Moon, Mercury, Venus, Earth and Io; the note's section 5 has the numbers.
- Every paper figure is from memory and unverified: the Consensus search is out until 1 Dec 2026 (the note's section 15 lists about fifty sources to read).
- Earth comes out right to about a third with nothing tuned. Mars's field date holds (the lid no longer takes its heat from the deep mantle) but sits near the guard's old edge. Lid worlds make 2 to 5 times too much crust, because the kernel's lid mantles run hot (two existing fudge rows). Io's and Enceladus's heat worked out from their interiors alone will be one to two orders low until the orbit side (HF6) is designed.

| Item | State |
|---|---|
| The note in `handover/design/`, this file, the row in START-HERE | done, 26 Nov |
| Scratch model in project files `sse4/melt-proto/` | done, 26 Nov |
| Alex's answers to questions 1 to 4 | waiting |
| Coordinator's answers to C1 to C7 | waiting |
| Read the sources at source and correct the sheet rows | from 1 Dec 2026, citations thread (C7) |
| HF6 the orbit side (host's tide, resonance partners, eccentricity that dies away): its own design | written as a proposal 26 Nov at the coordinator's word: `design/SSE4-tidal-orbits-design.md`, state in `status/tidal-orbits.md` |
| HF4d ice shells: a short note from its builder | after HF4c.2 |

Packages (owner: the heat flow thread; states live in `status/heat-flow.md`): HF4b.0 numbers and blocks, HF4b.1 the lid, HF4b.2 melt, HF4c.1 tides read the state, HF4c.2 layers, then HF2, HF5, HF4d. Core golden lane: HF4b.1, HF4b.2, HF4c.1, one turn each.
