---
name: sse4-new-physics-demo
description: Video G "Teach it new physics" (Orac brings in a paper as a pack), track ND ACCEPTED 22 Nov 2026; feasibility findings and Alex's framing
metadata:
  type: project
  modified: 2026-11-22T13:25:14.986Z
---
Alex, 22 Nov 2026: one more video, made near the end when most visual effects exist. Orac takes in a paper, expresses it as visible, editable, cited blocks, saves it as a pack others load with no key ("only one user pays"). Test paper: coloured white dwarfs, Hollands et al. 2026, doi:10.1093/rasti/rzag069 (CC BY 4.0). His framing: "no-one else can do this", it makes SSE usable for academic what-ifs, and the "killer blow" is that the paper is surfaced as simple blocks, not hidden in a codebase (the central shot).

Plan: /mnt/project-files/sse4/SSE4-new-physics-demo.md (repo copy handover/design/, PR 170), build plan section ND and milestone M9, STORYBOARDS.md section G. Thread "Plan Orac new-physics demo video" (Fable).

Findings at main 2cef498: not possible yet and not with zero new blocks. Colour science (CIE to sRGB) already exists in ui-contract light.ts. Missing: white dwarf atmosphere facts (B3), broad absorption in a spectrum and spectrum on a wire (with O4), maths beyond arithmetic in boxes, Orac paper intake, Orac adding and wiring nodes and assembling a set, packs carrying their own fudges, sources and reproduction report, export of Orac's work as a pack, Explorers publish (X2). Proposed honest claim: "no new code for this paper".

Alex 22 Nov 15:23: question 1 accepted, the film's claim is "no new code for this paper" ("blocks are NOT code"). Questions 2 to 7 still open; the repo copy of the design gets synced once more answers are in.

Alex 22 Nov 15:29: a star's broad absorption is the same mechanism as a planet's air having a colour. ND2 is one "light through gas" block, per-gas absorption rows on the data sheet, no body type named (design section 2.6, PR 184). His standing ask from this: in design "think wide" for other use cases so a capability is extensible and rework is avoided, in line with processes, not classes.

Alex 22 Nov 15:30: question 2 accepted ("blocks should be able to do basic math... or just have a general purpose equation, which is what I thought we had"). Design /mnt/project-files/sse4/SSE4-equation-block-design.md: maths blocks plus an Equation block (formula as pack data, units checked on load, constants only via Look up, sourced, worked examples). ND3 handed to "Build kernel resolve and rules" 22 Nov. Three small questions in its section 9 open. Questions 3 to 7 of the demo plan still open.

Alex 22 Nov 15:56: yes to moving existing atoms into Equations gradually (ND3b), each with a name that means something (design section 8A). Equation questions 1 (bare numbers) and 3 (name "Equation") still open.

Alex 22 Nov 16:02: every letter in a formula has a visible line from a source: dynamic (another block), reference (Look up on the data sheet) or a Model number block (sourced, graded); no parameters inside an Equation; small numbers (1, 1/4) are just the formula (bare-number rule accepted). Timing rule for ND3b: within 5% (my figure, open). Still open: the name "Equation", demo questions 3 to 7.

Alex 22 Nov 16:03: ALL questions accepted (demo 1 to 7; Equation: bare-number rule, the name "Equation", 5% timing rule). Packages handed to owner threads 22 Nov. Authors: Alex writes near filming time, once there is a rough result to show; spectra sent on request are NOT under the paper's CC BY licence, so anything derived from them ships in a pack only with the authors' explicit permission.

**How to apply:** ND is accepted; build to SSE4-new-physics-demo.md section 3 and SSE4-equation-block-design.md. B1/B3 build atmosphere facts only and leave coloured white dwarfs for Orac to do on camera. ND2 is one "light through gas" block for every body. Equations: every letter wired from a dynamic value, a Look up or a Model number block; names in plain words. Never stage the film: real key, real cost. Related: [[agents-track]], [[sse4-trailer-capture]], [[star-types-and-one-body]]
