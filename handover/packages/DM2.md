# DM2: dark energy's pressure in the expansion and the growth of structure

Track GH (`design/SSE4-galaxy-histories-design.md`, section 3.4), built 23 Nov 2026 by the kernel thread.

## What it does (`crates/sse-cosmos/src/cosmology.rs`)
- `Params` gains `w0` and `wa`, read from the data sheet's Cosmos rows (DM1, marked theoretical), w = w0 + wa (1 − a) (Chevallier and Polarski 2001; Linder 2003). They serialise only when not −1 and 0, and a cosmological constant takes exactly the old code path, so no golden moves.
- `dark_energy(a)` = a^(−3(1 + w0 + wa)) e^(−3 wa (1 − a)), in `e(a)` and so in cosmic time, distances, the web and every halo.
- With w other than −1 the growth factor is stepped from its equation, D'' + (2 + d ln H / d ln a) D' − (3/2) Ωm(a) D = 0 in ln a, fixed RK4 from a = 10⁻⁴ (2,000 steps, tabulated once per cosmology, under a millisecond). For Λ, Carroll, Press and Turner as before; `growth_by_steps` gives the stepped one for checks.
- No fudge: dark energy is in the expansion and the growth, its honest place; nothing pushes on voids.

## "Real size"
The web slice already carries its `scale_factor`; a map drawn in physical megaparsecs multiplies positions and the extent by it. The switch itself is the galaxy charts thread's (told 23 Nov).

## Checks (`crates/sse-cosmos/src/dm2_tests.rs`)
- Stepped growth against Heath's exact integral for Λ: within 0.2 % from a = 0.05 to 1 (the design's check).
- Growth rate today against Linder's growth index (Linder 2005, doi:10.1103/PhysRevD.72.043529) within 1 % for w = −0.8, −0.9, (−0.9, −0.3) and (−1.1, 0.2). For Λ the index comes out 0.5545. DESI's (w0, wa) = (−0.75, −0.86) crosses w = −1, where the fit does not hold (0.557 here against its 0.546); its growth is checked to rise smoothly to today instead.
- Thinner dark energy (w = −0.8) stalls growth earlier and gives a younger universe at the same H0.
- The web's voids (stacked around the 40 deepest minima of a 320 Mpc slice today): an emptied middle, density climbing steadily to the mean 10 to 40 Mpc out, still below −0.3 half-way. Against Hamaus, Sutter and Wandelt 2014 (doi:10.1103/PhysRevLett.112.251302) the middles are emptier (δ about −0.99 against their −0.8 to −0.9): the web's lognormal density (already on the register) empties its minima fully. The compensating wall is weak (about +0.1); not asserted.

## Next
DM3 (matter ledger per cell and per halo, op `matter_budget`).
