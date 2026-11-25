# Status: the history of spaceflight as a view, and In memory (track SH)

Owner thread: none yet for the build. To be opened as "Build roll call and In memory" (Opus), owning `shell/src/rollcall/` (accepted, question 13). Design thread: "Design spaceflight history view and memorial" (Fable).
Design: `design/SSE4-spaceflight-history-design.md`, **ACCEPTED by Alex 25 Nov 2026 01:20 UTC ("yes to all - stay restrained and respectful - one place we don't joke")**. Read its sections 1, 3, 8.1 and 14 first; packages are its section 13. Mockups are in project files at `sse4/handover-screens/SH-screens/`.

Nothing is built yet. All 13 recommendations in section 14 are rulings. SH1 and SH2 can start as soon as RS9 part 1 (`store-packs/sol-today/crews.json`) is on main.

| Package | State | Log |
|---|---|---|
| Design note and mockup | done 25 Nov, accepted | |
| SH1 `rollCall(campaign, t)` | next; needs RS9 (people and moves) for the real pack | |
| SH2 Roll call screen and rail entry | queued | |
| SH3 the count strip on the ruler | queued; with the chart room | |
| SH4 a person's flights; Find knows people and dates | queued; with the manifest thread | |
| SH5 records as sifters; On this day | queued | |
| SH6 In memory; the day of a loss | queued; needs deaths and `how: "lost"` from the pack | |
| SH7 Names carried (real memorials) | queued; needs `memorials.json` from the pack | |
| SH8 front door address and deep links | queued; chart room | |
| SH9 tours, record sifters, the edge of space | queued; with the spaceflight pack thread | |

Data asked of the spaceflight pack thread (design section 10): deaths with `during` and two sources, people with no flight (training and ground deaths), `how: "lost"`, the people-in-space step list, the edge of space as a pack value, `memorials.json`. That thread is building RS9 (people and moves) from McDowell's astronaut tables.

Standing rules for whoever builds this (design 8.1, accepted; Alex: "one place we don't joke"): every name and date sourced twice, nothing invented, no spectacle, plain words, no decoration, never a backdrop or a joke.
