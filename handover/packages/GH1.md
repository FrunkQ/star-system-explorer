# GH1: one assembly tree per halo

Track GH (`design/SSE4-galaxy-histories-design.md`, section 7.1), built 22 Nov 2026 by the kernel thread. It replaces Y4 part 2's separate satellite draw: the satellites are now the members of the host's tree.

## What it does (`crates/sse-cosmos/src/assembly.rs`, fudge F41)
- `assembly::tree(cosmology, seed, halo, z_read, params, level)` gives every halo the haloes it swallowed, largest first, each with its address (`…/prog:m{k}` for a pin, `…/prog:b{bin}-{i}` for a fill member), ψ, mass at infall, infall time and where its own keyed draws start.
- **Pins.** G2's major mergers before the time read are members first. FMBK's ξ is the smaller over the larger progenitor and its rate is per descendant mass, so the smaller one held ξ/(1+ξ) of the halo at the merger. (G2's chronicle text is unchanged.)
- **Fill.** Giocoli et al. 2008's unevolved subhalo mass function, in bins of ψ anchored at 10⁻³ (8 per decade), keyed Poisson counts on Y4's own stream and counters, each bin's mean less the pins that fall in it. Drawn top down.
- **Levels.** `Chart` reads to ψ 10⁻³, `Survey` to 10⁻⁵. Deeper bins only add members under the Chart ones, with counters after them, so a member never changes with depth (test `a_deeper_level_keeps_every_member_already_drawn`).
- **Closure.** A member is kept only if the tree has room for it and for the mass the fill expects under it, down to 10⁻⁵ whatever the level (so depth never changes the rule). The tree never holds more than its host.
- **Recursion.** A progenitor is a halo of its own: `assembly::mass_today_for` finds the median halo that had its infall mass at its infall, and `Haloes::median_at` grows it at the progenitor's address. Its tree is the same function read at its infall.
- Settings live in `HaloParams.assembly` (serde default, so packs without it keep working). `SatelliteParams` keeps only the processes acting on satellites (dynamical friction, orbit ranges, Einasto placement, quenching).

## What changed for satellites
- `satellites_seen` reads the tree at the time asked. A satellite's galaxy is now grown at the progenitor's address rather than at `halo:sat-{j}-{i}`, so the satellites' light and stellar mass are drawn afresh (same statistics). Fill members keep Y4's ψ, infall and orbit draws, except in bins a pin claims and where closure turns a member away.
- The walk's `satellites` entry adds `expected`, `accreted_share` and `pinned`.
- The kernel contract golden (`tests/golden/contract*`) is re-blessed on purpose: its sky response's one satellite keeps its place, distance and infall, but its address becomes `…/halo:milky-way-like/prog:b12-0`, its galaxy is redrawn (V 6.90, was 6.90; stellar mass 6.2 × 10⁷, was 6.8 × 10⁷ M☉), and the walk's `satellites` entry gains the new fields.

## Measured (default parameters, 10¹² M☉ median haloes)
- Chart: about 52 members (55 expected before closure). Survey: about 2,100 of 2,195 expected.
- Pins average about 0.3 of the host; in 2 hosts of 30, G2's major mergers alone sum to over 0.95 of the host and closure keeps under 10 Chart members. That is a G2 matter (its FMBK rate from z 6 is generous for one host), noted here for DM4 and GH2 rather than changed now.

## Next
DM4 (halo structure and rotation curve), then GH2 (orbits; the coordinator relayed the design thread's conditions: step from infall, step size a function of the body alone and sized near pericentre, one host table per time slice tested against the direct formulas).
