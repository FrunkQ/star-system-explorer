# Status: kernel hand-offs

Owner thread: "Build galaxy arm fix and atmosphere advice". Covers kernel packages handed off from the kernel
thread's queue ("Build kernel resolve and rules" keeps `orbits.rs`, `stability.rs`, `withdraw.rs`, the barycentre
work and the editing K-list). Only the owner thread edits this file. Index: `handover/START-HERE.md`.

| Package | State | Log |
|---|---|---|
| GH arm sign fix: arms trail (envelope arm phase and galaxy chart crests) | done 25 Nov, PR 463; the glow test's plane-over-pole margin is 0.5 mag until a Local Arm lands (then back to 1.0) | packages/GH3-leading-arm.md |
| GH3 leading arm: a dated `leading_arm` mark from a small backwards pass or meal; rarity check (1 in 100, Iye et al. 2019) | done 25 Nov, PR 463 | packages/GH3-leading-arm.md |
| ATMOS `atmosphere_advice`: the science expects (redox sets the outgassed mix), differs, clashes (E8) with lifetimes, hydrogen escape, one suggested batch; V3 air imported as hand-set | done 25 Nov, PR 463 | packages/ATMOS.md |
| V3 `bary:N` nodes onto the hierarchy on import (`adopt.rs`) | next, after the kernel thread's barycentre PR (BC1) merges | |
