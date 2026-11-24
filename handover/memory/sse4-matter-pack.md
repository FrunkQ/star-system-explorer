---
name: sse4-matter-pack
description: Master data sheet (matter pack) design, 21 Nov 2026: one pack for constants, elements, substances; go-ahead, track MP, open questions
metadata:
  type: project
  modified: 2026-11-21T18:48:43.142Z
---
Thread "Design shared elements and compounds table" (Fable, 21 Nov 2026). Design: /mnt/project-files/sse4/SSE4-matter-pack-design.md, repo copy handover/design/, track MP in the build plan.

Alex 21 Nov: "get the table - we were risking drift. A single user editable master data sheet makes sense"; universal constants go in it too, with a Constants page where people can change G and see what happens. ACCEPTED 21 Nov 20:51: "yes to all" on section 12 (now rulings). He added: units and conversion factors on the sheet beside Constants (section 7A.6: factors worked out from constants, add-only, in MP1c), and the periodic table coloured by nucleosynthesis source as the default lens, with origin stories per element as the place for teaching (he finds "a gold ring was made in a neutron star merger" mind-boggling; section 8.1; stories must say what is settled and what is argued). Next from the design thread: Reference screen mockup before MP10.

Shape: pack `packs/base-reference`, crate `sse-reference`; tables constants, reference objects, elements (all 118), isotopes, substances, mixtures; every value cited; blocks declare what they read and skip a substance lacking a property. Marks: real, authored (real value beside it), fictional, GM view only. Elements ride inside formation's six materials and are worked out on a click (`read_composition`); TG11's inventory is one block over that; W8's `sse.volatile.species` is a view of the table. Changed constants: each block is "follows", "estimated" (cited scaling law, ESTIMATED stamp) or "cannot follow" (tables computed with real constants); a reach report shows before accepting; what-if branch is the usual place. Fitted and tuned model numbers stay off the sheet (their packs, fudge register).

Packages MP0 audit (citations thread), MP1 schema, MP1c constants, MP2 data (kernel and citations threads), MP3 inside W8, MP4 light, MP5 life, MP6 formation and composition (proven core), MP7 galaxy, MP8 = TG11, MP9 fictional, MP10 Reference screen (chart room, mockup first by a Fable thread), MP11 Orac, MP12 packs and Explorers, MP13 and MP14 later.

**Why:** code audit at main 1f66399 found no shared table: water typed in five places, Earth radius 6378.1 vs 6371 km under one name, about 60 retyped constants.
**How to apply:** W8 and TG11 threads should build their tables in this schema's shape if they start before MP1 and MP2. Related: [[sse4-weather-review]], [[physics-thread-queue]], [[sse4-processes-not-classes]]

## Later rulings (21 to 22 Nov, detail in design file)
- Units as a setting, calendars (7A.7, 7B, MP15), counted measures vs drive ratings (7A.8, MP16), autopilot capability: all accepted. Reference mockup layout accepted 22 Nov 00:40; must work on phone and dark: https://claude.ai/artifact/7xpcYZrMxnmmQp6et5YvPV
- Ices (4.3A, MP17 to MP21) accepted; computed boundaries marked "computed" with source link. Look up block (3A, MP17a, kernel thread). MP22 typed copies: Alex's own yes incl. CI.
Build state: [[sse4-matter-build-state]]
