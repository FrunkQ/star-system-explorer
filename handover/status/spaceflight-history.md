# Status: the history of spaceflight as a view, and In memory (track SH)

Owner thread: "Build roll call and In memory" (Opus), owning `packages/shell/src/rollcall/` (accepted, question 13). Design thread: "Design spaceflight history view and memorial" (Fable).
Design: `design/SSE4-spaceflight-history-design.md`, **ACCEPTED by Alex 25 Nov 2026 01:20 UTC ("yes to all - stay restrained and respectful - one place we don't joke")**. Read its sections 1, 3, 8.1 and 14 first; packages are its section 13. Mockups are in project files at `sse4/handover-screens/SH-screens/`.

SH1, SH2 and SH6 are built (PR 510, merged 25 Nov 04:35 UTC). All 13 recommendations in section 14 are rulings. Rail note: Roll call is the system rail's 11th entry; `RAIL_MAX` is 11 for now on Alex's word (25 Nov 01:59 UTC, "we can overpack the UI a bit for now - there will have to be a rationalisation once everything is in place"), back to ten at that rationalisation. CI runs `tests/ui/rollcall.e2e.mjs chromium firefox` (Alex's yes, same message).

| Package | State | Log |
|---|---|---|
| Design note and mockup | done 25 Nov, accepted | |
| SH1 `rollCall(campaign, t)` | done 25 Nov, PR 510 | `packages/SH1-SH2.md` |
| SH2 Roll call screen and rail entry | done 25 Nov, PR 510; front door so far is `?spaceflight=<folder>` only | `packages/SH1-SH2.md` |
| SH3 the count strip on the ruler | queued; with the chart room | |
| SH4 a person's flights; Find knows people and dates | queued; with the manifest thread | |
| SH5 records as sifters; On this day | queued | |
| SH6 In memory; the day of a loss | done 25 Nov, PR 510 | `packages/SH6.md` |
| SH7 Names carried (real memorials) | part 1 done 25 Nov (read, orbits, In memory part); part 2 (chart layer, "tonight", naming dates) next | `packages/SH7.md` |
| SH8 front door address and deep links | queued; chart room | |
| SH9 tours, record sifters, the edge of space | queued; with the spaceflight pack thread | |

Data from the spaceflight pack thread (design section 10), all on main with PR 506: deaths with `during` and two sources, people with no flight (training and ground deaths), `how: "lost"`, the people-in-space step list, the edge of space as a pack value, `memorials.json`. Open items for later packages are in the SH1-SH2 and SH6 logs (campaign flight logs via `session.flightLog`, "Also in flight" in pack mode, the front door opening Sol).

Standing rules for whoever builds this (design 8.1, accepted; Alex: "one place we don't joke"): every name and date sourced twice, nothing invented, no spectacle, plain words, no decoration, never a backdrop or a joke.
