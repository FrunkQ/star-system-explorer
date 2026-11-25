---
name: sse4-atlas-and-editions
description: SSE4 track AT, the Atlas view with data-defined editions (Survey atlas, The Guide with jokes, Company report), print under the view, four starter interface kits; accepted 24 Nov 2026; build state
metadata:
  type: project
---
Alex, 24 Nov 2026 19:52 and 19:54 UTC: SSE3's textual reports and star atlas must come to SSE4. The star atlas is an interactive VIEW players step through, seeing only what they are allowed; "The Guide" is a version of it with "free added jokes"; others may be dry and corporate (a company report with the organisation's logo, or "crap ASCII art" from its name until one is added). Print sits UNDER the view: it prints the page showing, in that edition's style. Editions must be customisable or buildable; no editor UI needed, but a written guide so others or Orac can design them.

Design note: handover/design/SSE4-atlas-and-reports-design.md (ACCEPTED 24 Nov 20:14, notes in 14A); build state in handover/status/atlas.md.

In brief: one view `sse.view.atlas`; an edition (`sse.edition/1`) changes words, order, look, margin notes, letterhead and gaps, never a fact; gaps from template and audience, never the secret.

SSE3 source: see [[sse3-source-repo]].

**How to apply:** check whether Alex has answered before building; never design a separate report generator or a print-time theme picker.



Build state 25 Nov 12:15 UTC: AT15 part 2 merged (PR 565, ee0aa53): players' Atlas part in PLAYER_STACK, kit look+palette on role pages (shell/src/atlas/kitlook.ts), tests/ui/kits.e2e.mjs in CI. Left: other parts on KitControl (owners), players' View style, weak-device fallbacks; AT5/AT8 part 2 wait on kernel.

Alex's rulings in order (24 to 25 Nov): see [[sse4-atlas-rulings]].
