# GH0 (timing half): an assembly tree and satellite orbits in WebAssembly

Galaxy histories design (`design/SSE4-galaxy-histories-design.md`) package GH0, the kernel's half: time a throwaway tree and orbit run in WebAssembly, so measured costs replace the estimates in sections 9.1 and 10. Built 22 Nov 2026 by the kernel thread. The citation check and the galaxy type walk are the citations thread's half.

## What was timed
A throwaway crate (`GH0-bench/`, saved as `.txt` so no build or audit picks it up) compiled for `wasm32-unknown-unknown` at opt-level 3, run in Node 22 and in headless Chromium, at full speed and with Chromium's CPU throttled 4 times (the renderer's "Moto G4" phone stand-in). The machine is a 2.1 GHz Xeon cloud runner, so "laptop" here is about a mid laptop.

- **Tree.** Y4 part 2's draw: the Giocoli unevolved subhalo mass function as keyed Poisson counts in 8 bins a decade, each progenitor's infall by bisection on the host's growth. Chart level is ψ ≥ 10⁻³ (47 progenitors in this draw), Survey ψ ≥ 10⁻⁵ (2,207).
- **G3 per progenitor.** `galaxy::evolve` on a halo of the progenitor's mass.
- **Orbits.** One test body each: kick-drift-kick leapfrog, 14,000 fixed steps over 13.8 Gyr (1 Myr), in an NFW halo (10¹² M☉, c 10) plus a Hernquist bulge and a Miyamoto-Nagai disc, with Chandrasekhar drag (erf and exp at every step). That is the design's section 7.3 orbit with every step paying full price.

## Measured (ms)

| Piece | Node | Chromium | Chromium, CPU ÷4 |
|---|---|---|---|
| Cosmology setup (`Cosmology::new`) | 330 | 337 | 1,449 |
| Tree, Chart level, counts only | about 0 (326 with setup) | about 0 | about 0 |
| Tree, Survey level, counts only | about 0 (312 with setup) | about 0 | about 50 |
| G3 for each Chart progenitor (47) | 125 (2.7 each) | 96 (2.0 each) | 628 (13 each) |
| Orbits, 55 bodies × 14,000 steps | 330 | 272 | 1,204 |
| Orbits, 500 bodies × 14,000 steps | n/a | 1,991 | 8,495 |
| Orbits, 2,000 bodies × 14,000 steps | 7,639 | about 8,000 (scaled) | about 34,000 (scaled) |

One orbit step costs about 0.3 to 0.4 µs on the laptop and about 1.7 µs on the phone stand-in.

## What this changes in the design
- **The tree is free.** Counts and infall times at Survey level cost nothing next to everything else. The estimate "under 0.1 s at Survey" holds.
- **G3 per progenitor is about 2 ms on a laptop and 10 ms on a phone.** At Chart (about 55) that is 0.1 s and 0.5 s. At Survey the Sketch envelope is needed, as the design says.
- **Orbits are about 8 times dearer than estimated.** Section 10 said 30 million steps at Survey would be under 1 s compiled. They take about 8 s on a laptop and over 30 s on a phone. Chart (about 55 bodies) is 0.3 s and 1.2 s, which is fine once and stored.
- **Cutting the bill is straightforward, and GH2 should do it:**
  1. Step a body only from its infall, not from the Big Bang: about half the steps on average.
  2. Choose each body's step from its own orbital time (about 200 steps an orbit keeps leapfrog's energy error small). Outer orbits need far fewer than 14,000.
  3. Read the host's acceleration and drag from a radial table per stored time slice instead of erf, exp, ln and sqrt at every step.
  Together these should bring Survey to about 1 s on a laptop. That needs measuring again in GH2.
- **The cosmology setup is 0.3 s on a laptop and 1.4 s on a phone.** Every kernel op that builds a `Cosmology` pays it today. Caching it per stack is a cheap win for `sky_from_here`, `galaxy_chart` and the rest. It is noted here for the kernel's queue.

## Not done here
The two other GH0 parts belong to the citations thread: checking every from-memory citation in the design's section 12, and the type coverage table.
