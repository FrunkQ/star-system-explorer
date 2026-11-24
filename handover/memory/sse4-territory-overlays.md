---
name: sse4-territory-overlays
description: Territory overlays design (track OV, ACCEPTED in full by Alex 23 Nov 2026): map layers that colour places by who holds them, at every scale
metadata:
  type: project
  modified: 2026-11-23T16:03:28.213Z
---
Design file `/mnt/project-files/sse4/SSE4-territory-overlays-design.md` (repo copy `handover/design/`, PR 277), written 23 Nov 2026 in thread "Design territory overlays across all scales". Mockup artifact "SSE4 Territory overlays mockup": https://claude.ai/artifact/6fUw6HgRsmXVegPR1iMox9 (page and stills in `/mnt/project-files/sse4/OV-screens/`).

Status: **accepted in full**. Alex, 23 Nov 16:15: "yes to all" (one reply covering three open question lists; the coordinator read it as including these fifteen and told him so). Section 12 holds the rulings. In the build plan as track OV.

Alex's ask (23 Nov 15:48): an overlay for politics and the like on planet surfaces (countries) and galaxy maps (areas of interest); V3 did it with tags; redesign it so it works "across all boundaries".

What was accepted:
- A **map layer** colours places by any value with few answers (a relation to an actor, an exclusive tag group, a choice kept value, a banded fact, what an audience knows, a computed reach). Politics is the first layer.
- A **holding** is a dated record relation `holds` between a place address and an actor, status holds, claims or influences, optional extent (reach, band, outline). Shapes are worked out by each map from where the places are; only outlines are drawn.
- Inherit down the address tree, roll up when seen from further out ("divided" under a 60 percent share); the answer tree (actors nest) sets the level shown by zoom.
- Between stars: reach (default 4 pc) met midway; with a pack grid placed, borders follow hexes.
- "According to": as it is, as a power draws it, as the players know it.
- Extends OV1 (see [[traveller-overlay-contract]]): `areas` in `OverlaySpec` and `OverlayLayer`, palette slots, frames `body-latlon` and `web-mpc`. Packages OV2 to OV14 with suggested owners in section 11.
- Packs state territory by **claim rules** (cousins of dressing rules), arriving as ghost-border candidates.

Related: [[sse4-tags-design]], [[sse4-naming]] (its Local layer needs `who_holds`), [[sse4-view-styles]].
