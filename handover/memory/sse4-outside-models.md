---
name: sse4-outside-models
description: Track OM (outside models in flows, kernel as a published module): BANKED by Alex 24 Nov 2026, not accepted, nothing queued; revisit only with a real use case
metadata:
  type: project
---
Alex, 24 Nov 2026: wants an "API block" so a scientist can put a deep data model of their own into a physics flow; "just local machine is maybe the best security check"; "they can approach us to build a code block of it if needed" (so the outside route is the try-it-out route).

Design note: /mnt/project-files/sse4/SSE4-outside-models-design.md (thread "Design outside models for the kernel", Fable). Status BANKED, NOT ACCEPTED. Alex 24 Nov 20:40 UTC: "don't bother... let's find a proper use case first - bank the idea - store it away for the future. It's overcomplex and likely unneeded." Questions unanswered, no package queued, no builder starts. Repo copy in handover/design/ with a "Banked ideas" section in START-HERE (PR 484). **How to apply:** do not propose or build any of it unless Alex brings a concrete use case; my reading is that he wants a real user need before any API or plug-in surface.

Core of the banked proposal (my design, never accepted): the Outside model block never calls out; it reads recorded answers keyed by model, version and exact inputs (optionally on a declared grid, so a recording is a table being filled). The GM's app fills the recording by three routes: table file, "ask sheet" file round trip, live call to a companion on loopback. This is H9.2 Class B plus "results as data". A pack never holds an address. No live asks in generating flows or loops. Sending out is an effect (track J), never a flow. Kernel published as npm module plus `sse talk`, with stable/trial/internal per request. Packages OM1 to OM11 in section 13; needs a general Table block (OM1) that did not exist on main at 5bbc39a.

Related: [[sse4-new-physics-demo]].
