---
name: sse4-night-sky
description: SSE4 night sky design (track Y, accepted 21 Nov 2026): sky from any world, count tree, catalogue to resolve, Gaia licence ruling, package owners
metadata:
  type: project
  modified: 2026-11-21T16:10:17.559Z
---
- Design: /mnt/project-files/sse4/SSE4-night-sky-design.md, repo handover/design/ (PR 75 merged 21 Nov). Alex accepted it on 21 Nov; the section 12 recommendations are rulings. Track Y in the build plan replaces V7 and G7's catalogue part.
- Key rulings:
  - Fill stars use a count tree (luminosity classes, multinomial splits down to the G5 10 pc cells). Y1 re-blesses the G5 golden on purpose, and Y7a must first store promoted stars' full descriptions.
  - The fill sky is drawn for the observer's frame (a documented fudge).
  - Cataloguing promotes and writes a record entry, but does not form the system.
  - Players catalogue through a survey intent with GM approval.
  - Device caps: about 20k points on phones, 200k on desktops. The glow carries the rest.
- Gaia data is CC BY-NC 3.0 IGO (non-commercial; ESA page checked 21 Nov). Anything Gaia-derived goes only in the optional real-sky pack, never in the app. Alex did not say whether SSE4 will ever take money.
- Curated real set: CNS5 (5,931 objects within 25 pc), AT-HYG v4 (CC BY-SA 4.0, 2.5M+ stars), GCNS optional, UNGC (869 galaxies), OpenNGC (CC BY-SA 4.0). WDS and Exoplanet Archive terms are unchecked.
- Owners:
  - Kernel resolve and rules: Y7a, Y1, Y2 draw, Y4, Y7, Y8 kernel, Y0 field.
  - Light and nebulae: Y2 fluxes, Y3 glow, Y6 rules.
  - Renderer: Y5 cube map.
  - Galaxy charts and travel screens: Y5 Window, Y6 screen, Y9.
  - Chart room ("Rebuild the chart room's top level"): Y0 Oort shell drawing, Y7 button.
  - Converter: Y8 SIMBAD and exoplanet reuse.
- Alex, 21 Nov: the default Solar System data is V3's own, authored over time and possibly wrong (including its Oort "belt").

Related: [[sse4-light-track]] [[sse4-renderer-track]] [[converter-track]]
