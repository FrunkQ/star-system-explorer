---
name: 4x-pack-assessment
description: 19 Nov 2026 feasibility check of turning "On the Expansion of Space" (another author's umpired 4X) into an SSE4 pack; gaps found and recommendation
metadata:
  type: project
  modified: 2026-11-19T11:08:16.449Z
---

Alex asked (19 Nov 2026) whether Orac could build "that guy's" 4X game, "On the Expansion of Space" (Google Doc 1vjRg2Oe9yB0LqmD5hxWTs15q8hp2zp6tocuVw852vYU, readable via the /mobilebasic URL with WebFetch; docs.google.com is blocked for curl), as a downloadable pack.

The game: umpired, hex-based, three pillars (Fight, Build, Talk). Resources metal, bio, gas; rated regions on planets; structures, research tokens and tech tree; army units (rating 1 to 6) and navy ships (speed, hull, attack ratings, module slots); three orbit bands per body; star-type hazards; stealth and scanners. Unfinished: win conditions, large battles, diplomacy, two conflicting combat versions, much left to the umpire.

Claude's answer (design only, nothing built or changed): not possible with today's blocks, because SSE4 has no game layer. Already covered: map and region types from physics via classification, fog of war by seats and sealed seats, umpire as GM with intents and approval, dice as keyed draws, chronicle replay. Missing: stored game state (ledgers per empire, unit health, build queues), a turn engine with sealed simultaneous orders, logic and dice atoms, "effect" blocks that emit events and loop over collections, pack-defined entities and order intents, tech tree, and views (orders sheet, empire dashboard, umpire desk, turn report). Estimated about 10 packages, roughly Traveller-pack size plus a kernel piece.

Recommendation given: worth doing as a generic "game layer" after the Traveller pack (X3d already needs the first pieces: sensor-ladder rule and constructs), umpire-assist first, full auto-resolution later. Publishing the pack needs the other author's agreement, and his rules need finishing. 

Alex's reply (19 Nov 11:23): the game layer, a persistent player layer for RPGs (misc data on any object, political moves) and logic blocks (classifier, autopilot that scans, decides, moves, empties cargo) are one thing; blocks made of blocks as in Ovid; he is a strong believer in multi-use. He asked for the specs to be tweaked.

Done 19 Nov: new design file /mnt/project-files/sse4/SSE4-record-logic-programs-design.md (track J, stages J1 to J9, DECIDED: Alex accepted it and all five recommendations 19 Nov 11:31, ruling that the classifier is a 'unique', heavy block usable as a block but with its own editor, now section 3.5 'engine blocks'; he is keen on simple-rule ship and fleet conflict and autopilots doing pirate patrols and piracy, now worked example 6.1): the record (notes, kept values, actors), typed wires and logic atoms with one condition language shared with K8 predicates, boxes compiled by inlining, triggers and effects through one GM gate, programs as seats driven by a flow, optional turns. Handoff got pointers (5.4, 6.2, new 6.9, 7, 7A.6, 8.7, 13); build plan got track J, milestones M5c and M5d, X3d now needs J4 and J7. Synced to the repo by PR 10 (docs only, branch claude/project-thread-anwwhg). Kernel thread told to start J1 to J3; UI thread told about the intent registry (J5). Points of care for built code: flow doc hash must not change when optional fields are added; scenes must filter notes by audience; the intent list (closed TypeScript union in packages/ui-contract) must move into the kernel as a registry.

Related: [[sse4-design-sources]]

Secrecy (Alex, 19 Nov 11:55: "careful we don't transmit too much to clients"), now section 8A of the design: the universe is shared, the record is need-to-know. Record data is computed only on the GM machine and sent per seat as filtered disclosures (never raw events, never broadcast); dice use a "table secret" not the universe seed, with a hash commitment; packs may declare a campaign `competitive`, which requires sealed seats (GM override shows a banner).

PR 10 merged into main 19 Nov 13:18 UTC (merge commit b79076d): track J design with section 8A is in handover/design/. Alex ruled both 8A additions on 21 Nov 16:39 UTC ("Agree"): dice keyed on a GM-only table secret, and `competitive` packs require sealed seats; now rulings 7 and 8 in section 11.

Later syncs: PR 21 merged 19 Nov (nebulae and light, build plan, star census). 21 Nov: track J gained section 6.2 (programs and ships over a time jump: exact by leg, exact by decision budget, then estimated from measured cycles and stamped ESTIMATED with exact always offered; ships past a template's `service_life` retired with a note; only moving the campaign's now resolves programs) and stage J6b (needs J6, J7, T2 one clock). Synced with the top-level UI and onboarding design by PR 54, merged 21 Nov. `handover/design/interface-style.md` duplicates `handover/interface-style.md` (CLAUDE.md points at the latter; `sync-design.sh` recreates the copy). Evolution design left unsynced by this thread until its header shows rulings; its own thread owns that sync.

21 Nov, from the accepted tags design (SSE4-tags-design.md): record design gained 2.4 (a tag is a label only; honesty and source marks are called "marks"; `stock` and `extracted` kept kinds per resource on any address; conserving `transfer` effect, built as TG11; `economy.` prefix and `trade` tag group reserved for the 4.1 economy pack, design EC0 after TG11, TG9, J6, X3) and 8.1 (TG9: `ship.discipline` and `ship.readiness` kept kinds, status tags act only through visible triggers, drive ability from hardware; built by the Traveller pack thread). Synced by PR 81.
