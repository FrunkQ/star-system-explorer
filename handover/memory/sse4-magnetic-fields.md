---
name: sse4-magnetic-fields
description: SSE4 magnetic fields design (track MF, 22 Nov 2026): where it lives, status, key research findings, Field instrument proposal
metadata:
  type: project
  modified: 2026-11-22T13:04:29.648Z
---
Thread "Design magnetic fields from fundamentals" (22 Nov 2026).

Design: /mnt/project-files/sse4/SSE4-magnetic-fields-design.md (repo copy handover/design/, PRs 126 and 163, both merged). Track MF, MF0 to MF16; 4.0 scope MF0 to MF10 and MF14. **Accepted 22 Nov 01:06**, all 13 recommendations are rulings. Alex: "the shielding is an effect - not a switch ... earth field varies over history - so again - stories flow from these facts", hence section 4.2A: field history as dated record events, every effect computed at a date. MF16 folded into MP17, MP17a, MP1c, MP13, MP21; MF blocks read the sheet via sse.reference.lookup.

Mockup (5 boards: Earth, Neptune cutaway, Mars-like world across time, red dwarf, True style Field instrument): https://claude.ai/artifact/G2gxsfep39GiSH8UbNztRz

Built state at main 204e425: dynamo strength blocks exist and are tested but nothing feeds them, so no generated world has a field; NAV belts are a stand-in gated on mass >= 200 Earth masses; no overlay.

Design core: one "field ledger" per conducting layer (make = dynamo if magnetic Reynolds number passes about 50 with convected power; keep = ohmic decay; squeeze = flux carried with matter), same for planets, stars, remnants.

Research findings that set scope:
- Venus lacks a field because its core is not cooling, not slow spin (V3 was wrong).
- Whether a field protects an atmosphere is disputed: escape computed channel by channel.
- Strength predictable to about a factor of 2; shape "argued"; cycle lengths, reversal and flare timing drawn from measured statistics. Sun's 6-year cycle in O10 is the lower measured branch, not a bug.
- Ice giants: thin-shell default, Militzer 2024 as swappable rival.

Field instrument on True views (Alex asked 22 Nov 12:49 for an "instrument" showing fields as lines or halos): sections 9A, 9B, package MF14c, questions 14 to 16 **accepted 22 Nov 13:03** ("agree with your recs - it looks great - will look great moving in 3d"; PR 165). Field is an entry in the band switch whose reading replaces the visible picture (keeps "one picture at a time"); readings Lines, Halo (gpu-high), Surface. Alex 13:04: "the sensor package can be given to players like any other - defined by the 'lense'", so no special ship rule: a seat has the Field instrument when its lens (handoff 8.6, U9) includes it, and sees what it has measured. Renderer thread told to build MF14c after MF14a. Alex 13:04: "Perhaps 'Mag' instead of 'Field' in UI": the layer toggle and band switch entry are labelled **Mag** (design and mockup updated; kernel names unchanged).

Working notes: doi.org and Crossref are blocked from the cloud container; references marked (a) DOI seen, (b) title found, else from memory for MF0. Force-push of the thread branch is denied; restart a merged branch by merging the old remote head keeping main's content, then push normally.

Related: [[sse4-renderer-track]], [[sse4-processes-not-classes]]
