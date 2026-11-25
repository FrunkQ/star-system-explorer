---
name: sse4-from-here
description: SSE4 track FH "From here" (views and sounds from inside worlds): design note, mockups, state and Alex's brief of 25 Nov 2026
metadata:
  type: project
  modified: 2026-11-25T12:17:41.627Z
---

- Alex, 25 Nov 2026 01:56 to 02:01 UTC: redo SSE3's "what it looks like from here" far better ("make it BETTER!", "glorious"): down the levels of a gas giant as deep as practical; an eyes-adapted version and a true midday brightness version; marker posts, buoys and floaters to show how far you can see; a hand holding a colour card over the scene; standard clips played through a filter for that air, silence in vacuum. On planets these are **an option on the standing viewer**, not a separate screen. Venera (Venus) and Pluto (tholin) are his test cases.
- Note: `/mnt/project-files/sse4/SSE4-from-here-design.md`, repo `handover/design/` (PR 513), status `handover/status/from-here.md`. Mockups `sse4/handover-screens/FH-screens/` (13 screens, `from-here-mockup.html`). Design thread "Design views and sounds from inside worlds" (Fable).
- State: **ACCEPTED by Alex 25 Nov 2026 12:15 UTC** ("otherwise yup to your questions": yes to all 14; PR 571). His steers (note section 17A, which wins): aids are off by default ("uninterrupted view is better"), folded under the HUD chip, remembered at planet level; going down is seen as a sinking camera, never a jump; Listen's "Your own…" lists the sounds loaded for the Playbook and Switchboard. Acceptance on main (PR 571 merged 25 Nov 12:18 UTC). Nothing built; the builder thread "Build views and sounds from inside worlds" (Opus) is NOT open yet: opening it is the first thing to do on the new account, first slice FH5, FH6 (card rows), FH8, FH11, FH12. Open with Alex: does "remembered at planet level" mean per device (my reading) or per planet.
- Core ideas: one kernel op (working name `read_from_here`) for light and sound at a station anywhere in a fluid; Exposure gains "As lit" (screen white = Earth's noon) beside Long exposure (default) and Eye; Aids = Range markers, Colour card, Light meter, Listen; sounding line with named levels (Shirt-sleeve level, Last light, First glow, Checked physics ends); glow by Kirchhoff from the same absorption rows; card patches are data sheet substances; sound comes from the markers. Packages FH1 to FH17 by owner thread in section 12.
- Its citations (section 16) are from memory, unverified: Consensus sign-in had expired in that session.

Related: [[sse4-surface-descent]], [[sse4-visual-polish]], [[sse4-postcards]], [[sse3-source-repo]].
- Alex 25 Nov 12:10 UTC (clip ruling, in the note's Listen section, PR 569 merged): no film, TV or game clips however short or credited. Shipped clips are fresh recordings, public-domain SF text read fresh (his pick: the opening of Wells's War of the Worlds, the novel's words, not the musical or the 1938 broadcast), and NASA audio labelled with how it was recorded (prefer cabin recordings; radio clips are already narrow).
