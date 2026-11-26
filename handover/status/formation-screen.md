# Status: the Formation screen (F10, the reading half)

Owner thread: "Build the Formation screen" (Opus, opened 26 Nov 2026 on the new account). Design: `design/SSE4-system-formation-design.md` section 7 (accepted; Alex said yes again to building it on 26 Nov 2026, the "formation view" decision that had been open since the handover, pointing at the Kestrel mock-up "New system formation, the story of one star system"). On 23 Nov he agreed to build the reading half first and "Change the story" later.

Only the owner thread edits this file. Logs are in `handover/packages/`.

| Package | State | Log |
|---|---|---|
| F10a the chart: distance across, time down, a track per body (width by mass, bends as it migrates, joins at merging impacts, runs off the edge when thrown out), the gas disc and its condensation lines, belts, events, the upheaval span; the stage strip; the time slice tied to the time ruler; the Inherited panel and "Worked out at"; kernel op `formation_record`; rail entry "How it formed" (system, Understand, Explore and Build desks) | done in this PR | packages/F10a.md |
| F10b the selected world's story card (where its material came from, its events, how sure each number is), "How this was worked out", "Watch it form" as the screen's Play, opening from the Make a system preview | next | |
| Change the story, the Orac panel | later: needs the engine to branch a formation run from a stored moment (F9) | |
| Passing star and comet shower lines | later: needs systems tied to their place in the galaxy (star slots, GZ13 and GZ2) | |

## Found on the way, for its own job
- The system rail lists 13 entries on main with the Traveller pack (12 without), past `RAIL_MAX` 11 (`ui-contract/src/screen.ts`). Its only check, `tests/ui/constructs.test.ts`, counts a fixed list of screen documents and misses hazard, postcards and switchboard (and now formation). Not changed here.
