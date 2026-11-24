---
name: sse4-exogeology-review
description: 21 Nov exogeology gap review (Pluto etc.): where it lives, key findings, proposed packages S0b and S17 to S21, status
metadata:
  type: project
  modified: 2026-11-21T14:19:17.915Z
---
Thread "Research exogeology features we're missing" (21 Nov 2026). Report: /mnt/project-files/sse4/SSE4-exogeology-gaps.md; folded into the accepted Rock 3 proposal as section 5.9, order items 0b and 7 to 10 under question 2, questions 9 to 13, and build rows S0b, S17a/b, S18a/b, S19 to S21, all accepted by Alex 21 Nov; synced in PR 69 with tholin blocks (S19) and [[sse4-processes-not-classes]].

Findings from running the built code (main fc31eff; probe kept at /mnt/project-files/sse4/exogeology/pluto_probe.rs): Pluto, Charon, Triton, Ceres, Iapetus and Callisto all fall into the ice-shell regime (the rule is only water_fraction >= 0.05), so Pluto gets Europa-style ridges; Callisto gets 587 ridges and loses 96% of its craters; Enceladus and Miranda are "small body" (lumpy) because the roundness cut is 300 km for every material. No volatile ices (N2, CH4, CO, CO2) exist anywhere in the surface code.

Top recommendation: S0b small regime fixes, then S17 volatile-ice layer (a layer over all regimes, not a new regime), D1 equilibrium caps cheap, glaciers, convection cells and penitentes at D2 and D3.

Consensus: its sign-in is on Alex's company Claude account, not this one (ask him to run a prompt there). The 21 Nov results are kept at /mnt/project-files/sse4/exogeology/consensus-research-21nov.md and folded in as section 10 of the report; DOIs there are as given, not re-checked by the citations track yet.

Related: [[sse4-citations-and-fudges]]
