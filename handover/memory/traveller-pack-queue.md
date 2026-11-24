---
name: traveller-pack-queue
description: Traveller pack thread's done list and queue as of 23 Nov 2026 (gaps, TG, MP16, OV10, example packs)
metadata:
  type: project
---
Done by 23 Nov 2026 (all merged unless noted): X3a to X3f, TG9, TG13 universe packs (near-future, space-opera), TG8 part 1 (base-hooks access labels, optional intrigue pack), gap 13, gap 16 part 1 (ship's log), gap 26 tabletop embed (PR 265), Sector map on shared view controls (267), gap 25 brand and join words (272), gap 23 Describe via Minerva (273). PR 279: gap 25 live Hide labels and gap 18 sector grid overlay ([[traveller-overlay-contract]]).

Queue:
- TG8 part 2 plus TG11 kept half are OURS (inventory thread, 24 Nov): kept kinds extracted and stock, held = natural - extracted + stock, mining program step, resource.* label rules over held. Builds on PR 362 (MP8b): kinds in provides.other.resources.kinds [{id,name?,matter?,reach?,about?}]; read_inventory {system,body?,kinds?,at_s?}; ore deposits via survey_surface holdings (MP8a). MP8c merged 24 Nov (PR 386): reach "envelope" (kind matter h, reach envelope = gas skimming; rocky worlds 0); isotope kinds (matter he3) give real He-3 on all four giants. TG8 check now unblocked.
- MP16: pack declares counted measure "jump" ("2 jumps") and ship drive rating ("Jump-2", never "J2"); default jumps = ceil(route pc / rating), GM count overrules; templates move from traveller.jump to the rating. Needs kernel counted-measure declarations first (ask kernel thread).
- Gap 23 next: show the kept description on the card; players seeing it needs record notes to reach p2p replicas (chart room + kernel lane).
- Gap 25 next: Hide orbit lines, live switches for the single table and sealed seats, "size coming" on join.
- Gap 18 next: imported sector worlds tied to the stars under their hexes once the grid is placed.
- Gap 16 remainders: changed-field list on EventSummary (kernel), g-force and dose (travel thread).
- OV10 pack half (accepted 23 Nov, stage 5, after kernel claim rules): territory layers in space-opera and near-future packs; Traveller allegiance and travel zones as hex map layers. Design: SSE4-territory-overlays-design.md section 11.
- Future tech (FT, accepted 23 Nov, SSE4-future-tech-design.md): FT7 megastructures on that footing and FT8 Traveller TL table as a tech ladder, with the kernel thread; wait on FT1 (data sheet thread) and FT2 (kernel). Alex for every game pack: tech level (problems solved) right per universe and per faction, and how the rules handle FTL (jump routes, drive classes).
- Gap 15 megastructures (now FT7): design note late in stage 5.
- Coordinator 24 Nov: after mining and sector routes, RK12 Traveller mapping (weapons as effect at range), later our part of RK16 counters; design questions to thread "Bring Atomic Rockets into SSE4".
- Later asks of 23 Nov (difficulty ladders, links, SV7, RK11/12/16, manifest): [[traveller-pack-asks-23nov]].
- Route import as links (ours): travel thread's `set_link {id,a,b,name?,kind?,one_way?,nav?,tier?,audience?,writer?,reason?,source?,withdraw?,at_s}` landed 24 Nov (PR 377); read back `links_in {at_s?,audience?,history?}`; plan_voyage uses crew-known links. Import sector-file routes (trade/courier/jump lanes) through it, source = our pack/import, audience+tier for routes players don't know. Counted measures (MP16) go in nav.counted.
