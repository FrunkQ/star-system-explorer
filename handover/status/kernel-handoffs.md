# Status: kernel hand-offs

Owner thread: "Build galaxy arm fix and atmosphere advice". Covers kernel packages handed off from the kernel
thread's queue ("Build kernel resolve and rules" keeps `orbits.rs`, `stability.rs`, `withdraw.rs`, the barycentre
work and the editing K-list). Only the owner thread edits this file. Index: `handover/START-HERE.md`.

| Package | State | Log |
|---|---|---|
| GH arm sign fix: arms trail (envelope arm phase and galaxy chart crests) | done 25 Nov, PR 463; the glow test's plane-over-pole margin is 0.5 mag until a Local Arm lands (0.9 with the Local Arm: the plane is now 0.96 mag over the pole) | packages/GH3-leading-arm.md |
| GH3 leading arm: a dated `leading_arm` mark from a small backwards pass or meal; rarity check (1 in 100, Iye et al. 2019) | done 25 Nov, PR 463 | packages/GH3-leading-arm.md |
| ATMOS `atmosphere_advice`: the science expects (redox sets the outgassed mix), differs, clashes (E8) with lifetimes, hydrogen escape, one suggested batch; V3 air imported as hand-set | done 25 Nov, PR 463 | packages/ATMOS.md |
| V3 `bary:N` nodes onto the hierarchy on import (`adopt.rs`): the heaviest member stands for the pair, companions orbit it at the separation, no barycentre rows kept | done 25 Nov, PR 463 | packages/BC1-v3-barycentres.md |
| GH Local Arm: the home galaxy's arms from measured positions (Reid et al. 2019), one general measured-arms list in the envelope; chart `arm_names` | built 25 Nov; 26 Nov the dust cause: a bar sweeps the inner gas (rings), the bulge's stars in the star field and the glow, no green for new systems where the bar swept the gas. PR waits for the rings' chemistry retune (CC3, check set in SSE4-conserved-chain-checks.md): the glow lane check still fails on mid-disc metals | packages/GH-local-arm.md |
| GH age heights: stars' scale height by age through the count tree, the draw and the glow (young stars at the gas layer, old ones heated) | not started; written up 26 Nov for a fresh agent | packages/GH-age-heights.md |
