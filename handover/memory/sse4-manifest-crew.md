---
name: sse4-manifest-crew
description: Track MC (ship manifest and crew roster), built 23 Nov 2026: how people and cargo aboard are stored, and what is still open
metadata:
  type: project
  modified: 2026-11-23T20:34:29.617Z
---
- Alex 23 Nov 19:53: "do the manifest and crew work... that IS good"; SSE3 had crew and life support as text boxes, SSE4's is a general tracking tool; "it will get a makeover with aliens [4.1] and again in RPG [4.2]". Owner thread "Build ship manifest and crew roster"; status file handover/status/manifest.md.
- MC1 (PR 318, merged 24 Nov 00:40 UTC; card has one "Aboard" button with a Crew/Manifest switch, agreed with the chart room; `crewAboard(subject, roster)` is the one people-aboard count, agreed with the Traveller thread): people and cargo aboard are record actors at the universe root (kinds person, cargo) with an `aboard.on` ref, kept kinds in base pack `sse.base-manifest` (on the base stack). Life support rates per kind of being are pack data (human from NASA BVAD, unverified); a kind with no rates is named apart, never given a human's. Ageing stops in cold sleep (fudge). No kernel change.
- Six numbered questions asked 23 Nov ~19:58 (aboard as one kind of thing, not assumed human, ageing, life support from stores, damage via readiness only, players via Known to); built on the recommendations, answers pending.
- MC2 (PR 370, 24 Nov): Known to eye on each person and lot (Build/Run desks) plus "Show the crew/manifest to the table" as one set_known list with one Undo. New entries are as known as their place (Kestrel ladder: heard_of). Gotcha: known_to reads at the campaign clock, so set_known must be sent at campaignNow, not ws.now_yr (body card and Constructs still use ws.now_yr; chart room and Traveller threads told).
- Alex 24 Nov 14:00, desk rule: Explore and Run allow manifest, crew and fuel updates but never changes to the ship itself (facts, picture, 3D design); edit mode (Build desk) is where ship changes live, the UI is context sensitive. MC3 (PR 391): "Picture" on the card, the designed ship (ShipPlans, `locked` off Build) kept as construct.design, or the GM's own raster picture construct.picture (960 px, ~150 KB).
- 24 Nov: PR 391 merged (one Pictures panel views/pictures for ships, stations and later bodies; fuel gauges). MC5 (PR 423): ship's Log reads aboard moves; fuel as kept values fuel.propellant_t / fuel.jump_t (travel thread draws down with setKept). Rulings, open questions and next steps are written in handover/status/manifest.md (account handover 25 Nov).
- Open: a fuel level in the tanks drawn down by voyages (travel track); players seeing the picture. manifest.e2e.mjs is not in CI's e2e list (CI edits need Alex's yes); ship's log should include aboard moves.
Related: [[sse4-ship-design]], [[sse4-secrets-and-visibility]]
