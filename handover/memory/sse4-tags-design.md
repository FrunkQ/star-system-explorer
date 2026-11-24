---
name: sse4-tags-design
description: SSE4 tags design (ACCEPTED by Alex 21 Nov): a tag is only a label; V3 tag mapping, TG1-TG15 packages, universe packs, economy 4.1, where V3 source lives
metadata:
  type: project
  modified: 2026-11-21T16:09:59.278Z
---
Design file /mnt/project-files/sse4/SSE4-tags-design.md, written 21 Nov 2026 in thread "Pare back and map V3 tags". Status: proposed, Alex has not ruled yet (nine questions in its section 8, each with a recommendation). Not yet synced to handover/design/.

Core proposal: in SSE4 a tag is only a label on an address or a note. No value, never read by physics, two sources (derived by pack rule, or authored), authored true/false overrules a rule tag, packs declare tags in `provides.other.tags` and an undeclared reference fails the stack load. V3's physical-description tags become facts and class atlas words; game and story tags stay; numbers V3 hung on tags (readiness, tardiness, richness) become kept values or facts. Packages TG1 to TG10 with owners are in section 7.

Facts worth not rediscovering:
- V3 source on AC-ProArt is C:\Development\star-system-explorer-v2\star-system-generator (not a folder named SSE). Tag code in src/lib/tags/, docs in docs/tags-guide.md, docs/classification-and-tags.md, docs/dev/engine-map.md (TAG-1 to TAG-25).
- Real V3 saves never carry the tag `origin` field (V3 infers it from flags and namespace). SSE4's importer filters on `origin` only, so it currently keeps every V3 tag in one opaque `v3.tags` fact that nothing reads.
- In the designs "tagged authored-impossible" and "source tag traveller-generated" are marks on values, not tags; the tags design asks to call them marks.

See [[sse4-build-conventions]].

Later on 21 Nov (Alex): resources are real inventories from physics with `resource.*` as a derived, survey-gated label on top; made and imported goods use the same inventory through `stock` on any address (design sections 5.5, 5.6; package TG11). Economy is a separate pack and a **4.1 feature** (after 4.0): placeholders now (in TG11, TG9, TG1), high-level design EC0 once TG11, TG9, J6 and X3 have landed. Build plan (project-files copy) has track TG (proposed) and track EC plus a "4.1 Economy" milestone row.

Also 21 Nov (Alex): tag visibility is just the usual data view filters, no separate mechanism (settled; only the default tier for a new GM tag is open). Universe packs are in V1: a setting pack adds its constructs and bases to worlds that fit via "dressing rules" (deterministic candidates, GM adopts or holds off), design section 5.7, packages TG12 to TG15 including pack shelves on the Explorers site. Rights caution raised by Claude and the coordinator, not ruled by Alex: ship only original, generic, public-domain or licensed settings; named owned settings user-made only.

Ruled by Alex 21 Nov (tags design question 10): built-in and featured universe packs stay original and generic; named-setting packs are allowed on Explorers as fan-made and unofficial (rules and plain names only, no logos, art or copied text, free), using the site's existing attribution and report mechanism, not a new one. Questions 1 to 9 still open.

Paid packs (21 Nov): Alex wants a charging mechanism on Explorers (he keeps 20%), designed in thread "Design paid packs on Explorers", not surfaced in the UI yet. Tags design 5.7 fixes three rules for it: named-setting fan packs can never be paid; paid packs must be original or rights-held; fan shelf kept apart from priced items. These rules were written by Claude at the coordinator's request, not explicitly ruled by Alex.


ACCEPTED 21 Nov 16:38 (Alex: "Agree with all tag recs after read through"): every recommendation in section 8 is a ruling; earlier lines here saying proposed or open are superseded. Synced to the repo in PR 80 (merged to main, 4bd610f): handover/design/SSE4-tags-design.md, build plan tracks TG and EC, START-HERE row. Owner threads told their packages: kernel TG1-5 and TG12; proven core TG6 and TG11 inventory; Traveller TG8, TG9 build, TG11 kept kinds, TG13; chart room TG10, TG14; converter TG7; Explorers TG15; story-layer design TG9 design; paid-packs thread carries the three paid rules for Alex's own review.
