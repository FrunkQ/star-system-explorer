---
name: sse4-surface-descent
description: Alex's 23 Nov ruling on flying to a planet surface (autoroute to a picked site, planet-bound free view) and landing facilities as styled procgen constructs
metadata:
  type: project
---
Alex, 23 Nov 2026 (relayed by coordinator, renderer thread owns building it):
- Flying to the surface: the player picks a destination on the planet and the ship autoroutes there (no hand piloting, as ruled before), using the kernel's planned descent.
- Once down: free view to wander and explore, kept planet-bound. The only way to see an off-planet sequence is to get back in the ship and chart a new course.
- Easiest pick (Alex 23 Nov 11:52): a landmark from a map, natural (peak, crater, coast) or construct; a press anywhere else still works.
- Land ANYWHERE (Alex 23 Nov 11:51), not only ports or cities: facilities are places that exist, not the allowed list; a rough site is offered with its dangers shown (danger is drama), never filtered out.
- Landing facilities go in the render catalogue: simple geometric, procgen so each is deterministically different, with styles per nation or species exactly like ships. Ground structures are constructs too, so ONE style mechanism covers ships and ground constructs (not two).
- Renderer order after this (23 Nov): True reachable in a press or two; designed ship on Voyages At world card plus "Watch the pass" (aerobrake glow); True galaxy 3D; surface (finer orbit globe, ground streamed by altitude, standing on surface, descent); cosmic web 3D; construct styles per nation last.
Why: Alex wants to see the renderer and fly down; ships and bases share one construct look.
How to apply: design the descent as pick-site -> kernel route -> flown camera -> Site-level free view; generalise base-ships `ship_styles` into construct styles used by facilities too.
Related: [[sse4-ship-design]] [[sse4-renderer-track]]
Accepted 23 Nov 11:53 ("yes to all"): (1) landmarks+map from atlas and naming packs, per-site ship verdict, skippable flown descent; (2) wander a few-km patch on foot or low drone, further = take off and re-pick; (3) facilities only where pack/story/GM places them, pads sized from ships served; (4) ship_styles become construct styles for ships and ground; (5) phones walk at gpu-basic plus Take photograph. Recorded in SSE4-surfaces-and-rendering.md 4.1.
